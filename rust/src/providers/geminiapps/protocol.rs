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

pub(super) async fn fetch(cookie: &str, timeout: u64) -> Result<UsageSnapshot, ProviderError> {
    let client = crate::core::credentialed_http_client_builder()
        .redirect(reqwest::redirect::Policy::none())
        // Gemini's bootstrap can reject this client's HTTP/2 streams.
        .http1_only()
        .timeout(std::time::Duration::from_secs(timeout.clamp(5, 60)))
        .user_agent("Mozilla/5.0")
        .build()?;
    let page = client
        .get(format!("{ORIGIN}/app"))
        .header("Cookie", cookie)
        .send()
        .await?;
    let html = read_body(page, BOOTSTRAP_LIMIT).await?;
    let csrf = parse_csrf(&html)?;
    let response = client
        .post(format!("{ORIGIN}/_/BardChatUi/data/batchexecute"))
        .query(&[("rpcids", RPC), ("source-path", "/app"), ("rt", "c")])
        .header("Cookie", cookie)
        .header("Origin", ORIGIN)
        .header("Referer", format!("{ORIGIN}/app"))
        .header("X-Same-Domain", "1")
        .form(&[
            ("f.req", "[[[\"jSf9Qc\",\"[]\",null,\"generic\"]]]"),
            ("at", csrf.as_str()),
        ])
        .send()
        .await?;
    parse_response(&read_body(response, QUOTA_LIMIT).await?)
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
    html.split_once("\"SNlM0e\"")
        .and_then(|(_, tail)| tail.trim_start().strip_prefix(':'))
        .and_then(|tail| {
            serde_json::Deserializer::from_str(tail.trim_start())
                .into_iter::<String>()
                .next()
                .and_then(Result::ok)
        })
        .filter(|s| !s.is_empty())
        .ok_or(ProviderError::AuthRequired)
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
    fn csrf_is_decoded_and_absence_requires_login() {
        assert_eq!(parse_csrf(r#"{"SNlM0e":"a\u003db"}"#).unwrap(), "a=b");
        assert!(matches!(
            parse_csrf("<html></html>"),
            Err(ProviderError::AuthRequired)
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
