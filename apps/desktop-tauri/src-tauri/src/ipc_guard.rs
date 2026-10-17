//! Remote sign-in pages must never inherit the desktop application's commands.
pub(crate) fn guard<F>(handler: F) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync
where
    F: Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync,
{
    move |invoke: tauri::ipc::Invoke<tauri::Wry>| {
        let webview = invoke.message.webview_ref();
        if !webview
            .url()
            .is_ok_and(|url| allowed(webview.label(), &url))
        {
            invoke
                .resolver
                .reject("Commands are unavailable from this webview");
            return true;
        }
        handler(invoke)
    }
}

pub(crate) fn allowed(label: &str, url: &tauri::Url) -> bool {
    matches!(label, "main" | "settings" | "floatbar" | "flyout")
        && matches!(url.scheme(), "tauri" | "http" | "https")
        && matches!(url.host_str(), Some("localhost" | "tauri.localhost"))
        && url.username().is_empty()
        && url.password().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_commands_require_both_a_native_surface_and_a_local_page() {
        for label in ["main", "settings", "floatbar", "flyout"] {
            assert!(allowed(label, &"http://tauri.localhost/".parse().unwrap()));
            assert!(!allowed(
                label,
                &"https://gemini.google.com/".parse().unwrap()
            ));
        }
        for label in ["gemini-browser-123", "other", ""] {
            for url in ["http://tauri.localhost/", "https://gemini.google.com/"] {
                assert!(!allowed(label, &url.parse().unwrap()));
            }
        }
        assert!(!allowed(
            "settings",
            &"http://localhost.attacker.test/".parse().unwrap()
        ));
        assert!(!allowed(
            "settings",
            &"https://user:pass@localhost/".parse().unwrap()
        ));
    }
}
