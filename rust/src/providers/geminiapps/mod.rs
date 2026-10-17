//! Gemini Apps quota, independent of Gemini CLI and Antigravity.
pub mod desktop_session;
mod protocol;
mod session;
pub mod session_cache;

use crate::core::{
    FetchContext, ManualEmptyCookiePolicy, Provider, ProviderError, ProviderFetchResult,
    ProviderId, ProviderMetadata, SourceMode,
};
use async_trait::async_trait;

// A refresh must load the last committed cookies after the previous request
// finishes. Settings invalidation can otherwise start a second collector while
// the old network request is still rotating the same Google session.
static FETCH: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub struct GeminiAppsProvider {
    metadata: ProviderMetadata,
}

impl GeminiAppsProvider {
    /// Verify an explicitly imported session and return its renewed header.
    /// This value must remain inside the native credential storage boundary.
    pub async fn verify_session(cookie: &str, timeout: u64) -> Result<String, ProviderError> {
        let _fetch = FETCH.lock().await;
        Ok(protocol::fetch(cookie, timeout).await?.cookie_header)
    }

    pub fn new() -> Self {
        Self {
            metadata: ProviderMetadata {
                id: ProviderId::GeminiApps,
                display_name: "Gemini Apps",
                session_label: "Current",
                weekly_label: "Weekly",
                supports_opus: false,
                supports_credits: false,
                default_enabled: false,
                is_primary: false,
                dashboard_url: Some("https://gemini.google.com/app"),
                status_page_url: None,
                tertiary_label_key: None,
            },
        }
    }
}
impl Default for GeminiAppsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for GeminiAppsProvider {
    fn id(&self) -> ProviderId {
        ProviderId::GeminiApps
    }
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }
    fn available_sources(&self) -> Vec<SourceMode> {
        vec![SourceMode::Auto, SourceMode::Web]
    }
    fn manual_empty_cookie_policy(&self) -> ManualEmptyCookiePolicy {
        ManualEmptyCookiePolicy::FailClosedWeb
    }
    fn manual_cookie_precedes_token_account(&self) -> bool {
        true
    }
    async fn fetch_usage(&self, ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError> {
        if !self.available_sources().contains(&ctx.source_mode) {
            return Err(ProviderError::UnsupportedSource(ctx.source_mode));
        }
        let cookie = ctx.manual_cookie_header.as_deref().filter(|s| !s.trim().is_empty())
            .ok_or_else(|| ProviderError::BrowserSignInRequired {
                message: "Connect Gemini Apps in Settings → Providers. Close Gemini once before importing its session.".into(),
                sign_in_url: "https://gemini.google.com/app".into(),
            })?;
        let _fetch = FETCH.lock().await;
        let (header, mut epoch) = session_cache::header(cookie);
        let session = protocol::fetch_with_checkpoint(&header, ctx.web_timeout, |renewed| {
            if let Some(next) =
                session_cache::save_if_current(cookie, epoch, renewed).map_err(|_| {
                    ProviderError::Other(
                        "Could not save the renewed Gemini Apps session securely.".into(),
                    )
                })?
            {
                epoch = next;
            }
            Ok(())
        })
        .await?;
        session_cache::save_if_current(cookie, epoch, &session.cookie_header).map_err(|_| {
            ProviderError::Other("Could not save the renewed Gemini Apps session securely.".into())
        })?;
        // This RPC does not supply a stable Google account identity.
        Ok(ProviderFetchResult::new(session.usage, "web"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn missing_session_never_imports_an_ambient_account() {
        assert!(
            GeminiAppsProvider::new()
                .fetch_usage(&FetchContext::default())
                .await
                .is_err()
        );
        assert!(!GeminiAppsProvider::new().metadata().default_enabled);
        assert_ne!(
            ProviderId::GeminiApps.cli_name(),
            ProviderId::Gemini.cli_name()
        );
        assert_eq!(ProviderId::GeminiApps.cookie_domain(), None);
    }
}
