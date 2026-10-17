//! Google session renewal. Values stay in the collector and encrypted storage.
use crate::core::ProviderError;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant, SystemTime},
};

pub(super) struct SessionCookies(BTreeMap<String, String>);

impl SessionCookies {
    pub fn new(header: &str) -> Result<Self, ProviderError> {
        if header.len() > 64 * 1024 || header.bytes().any(|b| b < 0x20 || b == 0x7f) {
            return Err(ProviderError::AuthRequired);
        }
        let mut cookies = BTreeMap::new();
        for part in header.split(';') {
            let (name, value) = part
                .trim()
                .split_once('=')
                .ok_or(ProviderError::AuthRequired)?;
            if !super::desktop_session::valid_name(name) || !valid_value(value) {
                return Err(ProviderError::AuthRequired);
            }
            cookies.insert(name.to_string(), value.to_string());
        }
        Ok(Self(cookies))
    }

    pub fn header(&self) -> String {
        self.0
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn freshness(&self) -> Option<&str> {
        self.0.get("__Secure-1PSIDTS").map(String::as_str)
    }

    /// Accounts receives only the two parent-domain authentication cookies,
    /// never Gemini host cookies, analytics or another provider's credentials.
    pub fn rotation_header(&self) -> Option<String> {
        self.0.get("__Secure-1PSID")?;
        Some(
            ["__Secure-1PSID", "__Secure-1PSIDTS"]
                .into_iter()
                .filter_map(|name| self.0.get(name).map(|value| format!("{name}={value}")))
                .collect::<Vec<_>>()
                .join("; "),
        )
    }

    pub fn absorb(&mut self, response: &reqwest::Response, accounts: bool) {
        for cookie in response.cookies() {
            // Flatten only the root scopes supported by the explicit import.
            let domain = cookie.domain().map(|s| s.trim_start_matches('.'));
            let allowed = if accounts {
                domain == Some("google.com")
            } else {
                domain.is_none() || matches!(domain, Some("google.com" | "gemini.google.com"))
            };
            if !allowed
                || cookie.path().unwrap_or_else(|| {
                    response
                        .url()
                        .path()
                        .rsplit_once('/')
                        .map_or(
                            "/",
                            |(parent, _)| {
                                if parent.is_empty() { "/" } else { parent }
                            },
                        )
                }) != "/"
                || !super::desktop_session::valid_name(cookie.name())
                || !valid_value(cookie.value())
            {
                continue;
            }
            let expired = cookie.max_age().map_or_else(
                || {
                    cookie
                        .expires()
                        .is_some_and(|expiry| expiry <= SystemTime::now())
                },
                |age| age == Duration::ZERO,
            );
            if expired || cookie.value().is_empty() {
                self.0.remove(cookie.name());
            } else if (self.0.len() < 128 || self.0.contains_key(cookie.name()))
                && self.header().len() + cookie.name().len() + cookie.value().len() + 3 <= 64 * 1024
            {
                self.0
                    .insert(cookie.name().to_string(), cookie.value().to_string());
            }
        }
    }

    pub fn rotation_due(&self) -> bool {
        let Some(sid) = self.0.get("__Secure-1PSID") else {
            return false;
        };
        // A bounded, process-local throttle stores digests only. A restart
        // renews once; normal polling renews at most once per ten minutes.
        static ATTEMPTS: OnceLock<Mutex<BTreeMap<[u8; 32], Instant>>> = OnceLock::new();
        let Ok(mut attempts) = ATTEMPTS.get_or_init(Mutex::default).lock() else {
            return false;
        };
        let now = Instant::now();
        attempts.retain(|_, at| now.duration_since(*at) < Duration::from_secs(600));
        let key = Sha256::digest(sid.as_bytes()).into();
        if attempts.contains_key(&key) {
            return false;
        }
        if attempts.len() >= 32 {
            return false;
        }
        attempts.insert(key, now);
        true
    }
}

fn valid_value(value: &str) -> bool {
    value.len() <= 16 * 1024 && !value.bytes().any(|b| b < 0x20 || b == 0x7f || b == b';')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_sends_only_parent_authentication_and_never_logs_values() {
        let cookies = SessionCookies::new(
            "SID=private; __Secure-1PSID=primary; __Secure-1PSIDTS=stamp; host_only=secret",
        )
        .unwrap();
        assert_eq!(
            cookies.rotation_header().unwrap(),
            "__Secure-1PSID=primary; __Secure-1PSIDTS=stamp"
        );
        assert!(
            SessionCookies::new("NID=guest")
                .unwrap()
                .rotation_header()
                .is_none()
        );
        assert!(SessionCookies::new("bad\r\n=secret").is_err());
        assert!(SessionCookies::new("SID=bad\n").is_err());
    }
}
