use super::session::SessionCookies;
use crate::core::{ProviderError, RateWindow, UsageSnapshot};
use chrono::DateTime;
use serde_json::Value;

const ORIGIN: &str = "https://gemini.google.com";
const RPC: &str = "jSf9Qc";
const BOOTSTRAP_LIMIT: usize = 8 * 1024 * 1024;
const QUOTA_LIMIT: usize = 512 * 1024;

fn schema_error() -> ProviderError {
    ProviderError::Parse("Gemini Apps usage response is unavailable or has changed.".into())
}

pub(super) struct VerifiedSession {
    pub usage: UsageSnapshot,
    pub cookie_header: String,
}

pub(super) async fn fetch(cookie: &str, timeout: u64) -> Result<VerifiedSession, ProviderError> {
    fetch_with_checkpoint(cookie, timeout, |_| Ok(())).await
}

pub(super) async fn fetch_with_checkpoint(
    cookie: &str,
    timeout: u64,
    checkpoint: impl FnMut(&str) -> Result<(), ProviderError> + Send,
) -> Result<VerifiedSession, ProviderError> {
    let cookies = SessionCookies::new(cookie)?;
    let rotate = cookies.rotation_due();
    fetch_at_with_checkpoint(
        cookies,
        timeout,
        ORIGIN,
        "https://accounts.google.com",
        rotate,
        checkpoint,
    )
    .await
}

#[cfg(test)]
async fn fetch_at(
    cookies: SessionCookies,
    timeout: u64,
    origin: &str,
    accounts: &str,
    rotate: bool,
) -> Result<VerifiedSession, ProviderError> {
    fetch_at_with_checkpoint(cookies, timeout, origin, accounts, rotate, |_| Ok(())).await
}

async fn fetch_at_with_checkpoint(
    cookies: SessionCookies,
    timeout: u64,
    origin: &str,
    accounts: &str,
    rotate: bool,
    checkpoint: impl FnMut(&str) -> Result<(), ProviderError> + Send,
) -> Result<VerifiedSession, ProviderError> {
    // Budget the entire exchange, not three independent 60-second requests.
    // The desktop's outer deadline can otherwise cancel after rotation but
    // before the refreshed session reaches secure storage.
    tokio::time::timeout(
        std::time::Duration::from_secs(timeout.clamp(5, 60)),
        fetch_exchange(cookies, timeout, origin, accounts, rotate, checkpoint),
    )
    .await
    .map_err(|_| ProviderError::Timeout)?
}

async fn fetch_exchange(
    mut cookies: SessionCookies,
    timeout: u64,
    origin: &str,
    accounts: &str,
    rotate: bool,
    mut checkpoint: impl FnMut(&str) -> Result<(), ProviderError> + Send,
) -> Result<VerifiedSession, ProviderError> {
    let client = crate::core::credentialed_http_client_builder()
        .redirect(reqwest::redirect::Policy::none())
        // Gemini's bootstrap can reject this client's HTTP/2 streams.
        .http1_only()
        .timeout(std::time::Duration::from_secs(timeout.clamp(5, 60)))
        .user_agent("Mozilla/5.0")
        .build()?;
    if rotate && let Some(header) = cookies.rotation_header() {
        let previous_freshness = cookies.freshness().map(str::to_owned);
        // Protocol facts only; independently implemented. The explicit target
        // and no-redirect policy keep authentication on Google's own origins.
        match client
            .post(format!("{accounts}/RotateCookies"))
            .header("Cookie", header)
            .header("Origin", accounts)
            .header("Content-Type", "application/json")
            .body("[000,\"-0000000000000000000\"]")
            .timeout(std::time::Duration::from_secs(timeout.clamp(5, 10)))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                cookies.absorb(&response, true);
                if cookies.freshness().is_some()
                    && cookies.freshness() != previous_freshness.as_deref()
                {
                    // A later quota/network failure must not lose a successfully
                    // rotated authentication cookie. Initial connect uses a
                    // no-op callback until its quota has actually verified.
                    checkpoint(&cookies.header())?;
                } else {
                    tracing::warn!(
                        "Gemini Apps rotation did not issue a fresh authentication cookie"
                    );
                }
            }
            _ => tracing::warn!(
                "Gemini Apps session renewal was unavailable; verifying the current session"
            ),
        }
    }
    let page = client
        .get(format!("{origin}/app"))
        .header("Cookie", cookies.header())
        .send()
        .await?;
    cookies.absorb(&page, false);
    let html = read_body(page, BOOTSTRAP_LIMIT).await?;
    let csrf = parse_csrf(&html)?;
    checkpoint(&cookies.header())?;
    let response = client
        .post(format!("{origin}/_/BardChatUi/data/batchexecute"))
        .query(&[("rpcids", RPC), ("source-path", "/app"), ("rt", "c")])
        .header("Cookie", cookies.header())
        .header("Origin", origin)
        .header("Referer", format!("{origin}/app"))
        .header("X-Same-Domain", "1")
        .form(&[
            ("f.req", "[[[\"jSf9Qc\",\"[]\",null,\"generic\"]]]"),
            ("at", csrf.as_str()),
        ])
        .send()
        .await?;
    cookies.absorb(&response, false);
    if response.status().is_success() {
        checkpoint(&cookies.header())?;
    }
    let usage = parse_response(&read_body(response, QUOTA_LIMIT).await?)?;
    Ok(VerifiedSession {
        usage,
        cookie_header: cookies.header(),
    })
}

async fn read_body(mut response: reqwest::Response, limit: usize) -> Result<String, ProviderError> {
    let status = response.status();
    if status.is_redirection() || status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(ProviderError::AuthRequired);
    }
    if !status.is_success() {
        return Err(ProviderError::Other(format!(
            "Gemini Apps returned HTTP {}.",
            status.as_u16()
        )));
    }
    if response
        .content_length()
        .is_some_and(|len| len > limit as u64)
    {
        return Err(schema_error());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if chunk.len() > limit.saturating_sub(bytes.len()) {
            return Err(schema_error());
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| schema_error())
}

fn parse_csrf(html: &str) -> Result<String, ProviderError> {
    // Decode exactly the bootstrap JSON string; never log bootstrap HTML.
    html.match_indices("\"SNlM0e\"")
        .filter_map(|(index, key)| html[index + key.len()..].trim_start().strip_prefix(':'))
        .find_map(|tail| {
            serde_json::Deserializer::from_str(tail.trim_start())
                .into_iter::<String>()
                .next()
                .and_then(Result::ok)
                .filter(|s| !s.is_empty())
        })
        .ok_or_else(|| {
            if html.contains("accounts.google.com/ServiceLogin") {
                ProviderError::OAuthExpired(
                    "Gemini Apps session expired. Reconnect in Settings → Providers.".into(),
                )
            } else {
                schema_error()
            }
        })
}

fn parse_response(body: &str) -> Result<UsageSnapshot, ProviderError> {
    if !body.starts_with(")]}'") {
        return Err(schema_error());
    }
    let mut quota = None;
    for line in body.lines().skip(1) {
        let Ok(Value::Array(batch)) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        for record in batch {
            if record.get(0).and_then(Value::as_str) != Some("wrb.fr")
                || record.get(1).and_then(Value::as_str) != Some(RPC)
            {
                continue;
            }
            if quota.is_some() {
                return Err(schema_error());
            }
            quota = Some(
                serde_json::from_str::<Value>(
                    record
                        .get(2)
                        .and_then(Value::as_str)
                        .ok_or_else(schema_error)?,
                )
                .map_err(|_| schema_error())?,
            );
        }
    }
    parse_quota(&quota.ok_or_else(schema_error)?)
}

fn parse_quota(quota: &Value) -> Result<UsageSnapshot, ProviderError> {
    let windows = quota
        .get(1)
        .and_then(Value::as_array)
        .ok_or_else(schema_error)?;
    let (mut current, mut weekly) = (None, None);
    for item in windows {
        let kind = item
            .get(2)
            .and_then(Value::as_u64)
            .ok_or_else(schema_error)?;
        let (target, minutes) = match kind {
            1 => (&mut current, 300),
            2 => (&mut weekly, 10080),
            _ => continue,
        };
        if target.is_some() {
            return Err(schema_error());
        }
        let used = item
            .get(1)
            .and_then(Value::as_f64)
            .filter(|f| f.is_finite() && (0.0..=1.0).contains(f))
            .ok_or_else(schema_error)?;
        let stamp = item
            .get(3)
            .and_then(|v| v.get(0))
            .ok_or_else(schema_error)?;
        let seconds = stamp
            .get(0)
            .and_then(Value::as_i64)
            .filter(|s| *s > 0)
            .ok_or_else(schema_error)?;
        let nanos = match stamp.get(1) {
            None => 0,
            Some(value) => value.as_u64().ok_or_else(schema_error)?,
        };
        let nanos = u32::try_from(nanos)
            .ok()
            .filter(|n| *n < 1_000_000_000)
            .ok_or_else(schema_error)?;
        let reset = DateTime::from_timestamp(seconds, nanos).ok_or_else(schema_error)?;
        *target = Some(RateWindow::with_details(
            used * 100.0,
            Some(minutes),
            Some(reset),
            None,
        ));
    }
    let mut snapshot =
        UsageSnapshot::new(current.ok_or_else(schema_error)?).with_login_method("Gemini Apps");
    if let Some(weekly) = weekly {
        snapshot = snapshot.with_secondary(weekly);
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn window(kind: u64, fraction: Value) -> Value {
        json!([2400, fraction, kind, [[1800000000, 0]]])
    }

    #[tokio::test]
    async fn quota_failure_keeps_rotated_authentication_for_the_next_request() {
        let mut server = mockito::Server::new_async().await;
        let rotate = server
            .mock("POST", "/RotateCookies")
            .with_header(
                "set-cookie",
                "__Secure-1PSIDTS=new; Domain=.google.com; Path=/; Secure",
            )
            .create_async()
            .await;
        let page = server
            .mock("GET", "/app")
            .match_header("cookie", "__Secure-1PSID=primary; __Secure-1PSIDTS=new")
            .with_body(r#"{"SNlM0e":"csrf"}"#)
            .expect(2)
            .create_async()
            .await;
        let quota_path = "/_/BardChatUi/data/batchexecute?rpcids=jSf9Qc&source-path=%2Fapp&rt=c";
        let failed = server
            .mock("POST", quota_path)
            .with_status(503)
            .create_async()
            .await;
        let mut saved = None;
        let result = fetch_at_with_checkpoint(
            SessionCookies::new("__Secure-1PSID=primary; __Secure-1PSIDTS=old").unwrap(),
            5,
            &server.url(),
            &server.url(),
            true,
            |header| {
                saved = Some(header.to_owned());
                Ok(())
            },
        )
        .await;
        assert!(result.is_err());
        rotate.assert_async().await;
        failed.assert_async().await;
        assert_eq!(
            saved.as_deref(),
            Some("__Secure-1PSID=primary; __Secure-1PSIDTS=new")
        );
        failed.remove_async().await;
        let recovered = server
            .mock("POST", quota_path)
            .match_header("cookie", saved.as_deref().unwrap())
            .with_body(framed(&json!([2, [window(1, json!(0.1))]])))
            .create_async()
            .await;
        assert!(
            fetch_at(
                SessionCookies::new(saved.as_deref().unwrap()).unwrap(),
                5,
                &server.url(),
                &server.url(),
                false
            )
            .await
            .is_ok()
        );
        page.assert_async().await;
        recovered.assert_async().await;
    }
    #[tokio::test]
    async fn renewal_is_used_by_bootstrap_rpc_and_the_next_fresh_client() {
        let mut server = mockito::Server::new_async().await;
        let rotate = server
            .mock("POST", "/RotateCookies")
            .match_header("cookie", "__Secure-1PSID=primary; __Secure-1PSIDTS=old")
            .match_body("[000,\"-0000000000000000000\"]")
            .with_header(
                "set-cookie",
                "__Secure-1PSIDTS=rotated; Domain=.google.com; Path=/; Secure",
            )
            .with_header("set-cookie", "accounts_only=do-not-forward; Path=/; Secure")
            .create_async()
            .await;
        let bootstrap = server
            .mock("GET", "/app")
            .match_header(
                "cookie",
                "__Secure-1PSID=primary; __Secure-1PSIDTS=rotated; host_only=keep",
            )
            .with_header(
                "set-cookie",
                "__Secure-1PSIDTS=bootstrap; Domain=.google.com; Path=/; Secure",
            )
            .with_header(
                "set-cookie",
                "wrong=blocked; Domain=other.google.com; Path=/",
            )
            .with_body(r#"{"SNlM0e":"csrf"}"#)
            .create_async()
            .await;
        let quota_path = "/_/BardChatUi/data/batchexecute?rpcids=jSf9Qc&source-path=%2Fapp&rt=c";
        let quota = server
            .mock("POST", quota_path)
            .match_header(
                "cookie",
                "__Secure-1PSID=primary; __Secure-1PSIDTS=bootstrap; host_only=keep",
            )
            .with_header(
                "set-cookie",
                "__Secure-1PSIDTS=rpc; Domain=.google.com; Path=/; Secure",
            )
            .with_header("set-cookie", "host_only=; Max-Age=0; Path=/")
            .with_header("set-cookie", "scoped=blocked; Path=/private")
            .with_header("set-cookie", "implicit_path=blocked")
            .with_body(framed(&json!([2, [window(1, json!(0))]])))
            .create_async()
            .await;
        let session = fetch_at(
            SessionCookies::new("host_only=keep; __Secure-1PSID=primary; __Secure-1PSIDTS=old")
                .unwrap(),
            5,
            &server.url(),
            &server.url(),
            true,
        )
        .await
        .unwrap();
        assert_eq!(
            session.cookie_header,
            "__Secure-1PSID=primary; __Secure-1PSIDTS=rpc"
        );
        rotate.assert_async().await;
        bootstrap.assert_async().await;
        quota.assert_async().await;
        bootstrap.remove_async().await;
        quota.remove_async().await;
        let next_page = server
            .mock("GET", "/app")
            .match_header("cookie", session.cookie_header.as_str())
            .with_body(r#"{"SNlM0e":"next"}"#)
            .create_async()
            .await;
        let next_quota = server
            .mock("POST", quota_path)
            .match_header("cookie", session.cookie_header.as_str())
            .with_body(framed(&json!([2, [window(1, json!(0.1))]])))
            .create_async()
            .await;
        let next = fetch_at(
            SessionCookies::new(&session.cookie_header).unwrap(),
            5,
            &server.url(),
            &server.url(),
            false,
        )
        .await
        .unwrap();
        assert_eq!(next.usage.primary.used_percent, 10.0);
        next_page.assert_async().await;
        next_quota.assert_async().await;
    }

    #[tokio::test]
    async fn login_redirect_is_never_followed_and_failed_verification_returns_no_session() {
        let mut server = mockito::Server::new_async().await;
        let redirect = server
            .mock("GET", "/app")
            .with_status(302)
            .with_header("location", "/credential-trap")
            .with_header("set-cookie", "SID=unverified; Path=/")
            .create_async()
            .await;
        let trap = server
            .mock("GET", "/credential-trap")
            .expect(0)
            .create_async()
            .await;
        let mut saved = false;
        assert!(
            fetch_at_with_checkpoint(
                SessionCookies::new("SID=original").unwrap(),
                5,
                &server.url(),
                &server.url(),
                false,
                |_| {
                    saved = true;
                    Ok(())
                }
            )
            .await
            .is_err()
        );
        assert!(
            !saved,
            "an unauthenticated login page must not replace the saved session"
        );
        redirect.assert_async().await;
        trap.assert_async().await;
    }
    fn framed(quota: &Value) -> String {
        format!(
            ")]}}'\n\n123\n{}\n",
            json!([["wrb.fr", RPC, quota.to_string(), null]])
        )
    }
    #[test]
    fn parses_zero_and_fraction_by_type_not_order() {
        let usage = parse_response(&framed(&json!([
            2,
            [window(2, json!(0.01084324)), window(1, json!(0))],
            true
        ])))
        .unwrap();
        assert_eq!(usage.primary.used_percent, 0.0);
        assert!((usage.secondary.unwrap().used_percent - 1.084324).abs() < 0.000001);
        assert_eq!(usage.primary.window_minutes, Some(300));
        assert_eq!(usage.primary.resets_at.unwrap().timestamp(), 1800000000);
        assert!(usage.account_email.is_none());
    }
    #[test]
    fn missing_weekly_stays_missing() {
        assert!(
            parse_quota(&json!([null, [window(1, json!(0.5))]]))
                .unwrap()
                .secondary
                .is_none()
        );
    }
    #[test]
    fn rejects_missing_invalid_duplicate_and_wrong_rpc() {
        for windows in [
            vec![],
            vec![window(2, json!(0))],
            vec![window(1, Value::Null)],
            vec![window(1, json!(-0.1))],
            vec![window(1, json!(1.1))],
            vec![window(1, json!(0)), window(1, json!(0))],
        ] {
            assert!(parse_quota(&json!([2, windows])).is_err());
        }
        assert!(parse_response(")]}'\n[[\"wrb.fr\",\"other\",\"[]\"]]").is_err());
        assert!(parse_response("<html>sign in</html>").is_err());
    }
    #[test]
    fn csrf_is_decoded_and_schema_failure_is_not_called_a_logout() {
        assert_eq!(parse_csrf(r#"{"SNlM0e":"a\u003db"}"#).unwrap(), "a=b");
        assert!(matches!(
            parse_csrf("<html></html>"),
            Err(ProviderError::Parse(_))
        ));
        assert_eq!(
            parse_csrf(r#"["SNlM0e"];{"SNlM0e":"valid"}"#).unwrap(),
            "valid"
        );
        assert!(matches!(
            parse_csrf("https://accounts.google.com/ServiceLogin"),
            Err(ProviderError::OAuthExpired(_))
        ));
    }
    #[test]
    fn rejects_invalid_reset_and_duplicate_batch() {
        let bad = json!([2, [[1, 0, 1, [[1800000000, 1000000000]]]]]);
        assert!(parse_quota(&bad).is_err());
        for nanos in [json!(-1), json!("bad"), json!(true)] {
            assert!(parse_quota(&json!([2, [[1, 0, 1, [[1800000000, nanos]]]]])).is_err());
        }
        let good = json!([2, [window(1, json!(0))]]);
        let frame = framed(&good);
        assert!(parse_response(&format!("{frame}{}", framed(&good))).is_err());
    }
}
