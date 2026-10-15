use super::*;
use codexbar::providers::geminiapps::{GeminiAppsProvider, desktop_session};

static MUTATION: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn changed(app: &tauri::AppHandle, header: Option<&str>) -> Result<(), String> {
    let pending = {
        let state = app.state::<Mutex<AppState>>();
        let mut state = state.lock().map_err(|_| "Usage state unavailable")?;
        codexbar::providers::geminiapps::session_cache::select(header)
            .map_err(|_| "Could not save Gemini Apps session securely")?;
        super::invalidate_account_usage(&mut state, ProviderId::GeminiApps)
    };
    crate::events::emit_provider_updated(app, &pending);
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = super::refresh_providers(handle).await;
    });
    Ok(())
}

/// A deliberate user action imports only Gemini's scoped session. Verify the
/// session before replacing the saved one; never send its values to the UI.
#[tauri::command]
pub async fn connect_gemini_apps(app: tauri::AppHandle) -> Result<(), String> {
    let _mutation = MUTATION
        .try_lock()
        .map_err(|_| "Gemini Apps connection is already in progress")?;
    let header = tauri::async_runtime::spawn_blocking(desktop_session::import)
        .await
        .map_err(|_| "Gemini session import failed")??;
    let header = GeminiAppsProvider::verify_session(&header, FetchContext::default().web_timeout)
        .await
        .map_err(|error| codexbar::logging::safe_error_message(&error))?;
    changed(&app, Some(&header))
}

#[tauri::command]
pub async fn disconnect_gemini_apps(app: tauri::AppHandle) -> Result<(), String> {
    let _mutation = MUTATION
        .try_lock()
        .map_err(|_| "Gemini Apps connection is already in progress")?;
    changed(&app, None)
}
