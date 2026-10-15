//! Renewal is separate from the selected login. Its file must not change the
//! mobile bridge's account-selection stamp on every successful quota refresh.
use crate::settings::ManualCookies;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

static MUTATION: Mutex<u64> = Mutex::new(0);

#[derive(Serialize, Deserialize)]
struct SavedSession {
    binding: [u8; 32],
    cookie_header: String,
}

fn path() -> anyhow::Result<PathBuf> {
    crate::settings::config_store_dir()
        .map(|root| root.join("geminiapps_session.json"))
        .ok_or_else(|| anyhow::anyhow!("Gemini session storage unavailable"))
}

fn binding(header: &str) -> [u8; 32] {
    Sha256::digest(header.as_bytes()).into()
}

fn read() -> Option<SavedSession> {
    read_at(&path().ok()?)
}

fn read_at(path: &Path) -> Option<SavedSession> {
    let text = crate::secure_file::read_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

pub(super) fn header(selected: &str) -> (String, u64) {
    let Ok(epoch) = MUTATION.lock() else {
        return (selected.to_string(), u64::MAX);
    };
    (
        read()
            .filter(|saved| saved.binding == binding(selected))
            .map_or_else(|| selected.to_string(), |saved| saved.cookie_header),
        *epoch,
    )
}

pub(super) fn save_if_current(selected: &str, epoch: u64, renewed: &str) -> anyhow::Result<()> {
    let current_epoch = MUTATION
        .lock()
        .map_err(|_| anyhow::anyhow!("Gemini session storage unavailable"))?;
    save_at(
        &path()?,
        &ManualCookies::load(),
        *current_epoch,
        selected,
        epoch,
        renewed,
    )
}

fn save_at(
    path: &Path,
    cookies: &ManualCookies,
    current_epoch: u64,
    selected: &str,
    epoch: u64,
    renewed: &str,
) -> anyhow::Result<()> {
    if !selection_matches(current_epoch, epoch, cookies.get("geminiapps"), selected) {
        return Ok(());
    }
    let saved = SavedSession {
        binding: binding(selected),
        cookie_header: renewed.to_string(),
    };
    if read_at(path)
        .is_some_and(|old| old.binding == saved.binding && old.cookie_header == saved.cookie_header)
    {
        return Ok(());
    }
    crate::secure_file::write_string(path, &serde_json::to_string(&saved)?)?;
    Ok(())
}

/// Serialize explicit selection changes with renewal writes. A completed
/// disconnect removes both copies; late requests cannot restore either one.
pub fn select(header: Option<&str>) -> anyhow::Result<()> {
    let mut epoch = MUTATION
        .lock()
        .map_err(|_| anyhow::anyhow!("Gemini session storage unavailable"))?;
    let mut cookies = ManualCookies::load();
    match header {
        Some(value) => cookies.set("geminiapps", value),
        None => cookies.remove("geminiapps"),
    }
    cookies.save()?;
    *epoch = epoch.wrapping_add(1);
    match std::fs::remove_file(path()?) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn selection_matches(
    current_epoch: u64,
    request_epoch: u64,
    current: Option<&str>,
    selected: &str,
) -> bool {
    current_epoch == request_epoch && current == Some(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renewal_is_bound_to_exact_selected_session_not_another_login() {
        let saved = SavedSession {
            binding: binding("selected"),
            cookie_header: "renewed".into(),
        };
        assert_eq!(saved.binding, binding("selected"));
        assert_ne!(saved.binding, binding("reconnected"));
        assert_ne!(saved.binding, binding(""));
        // The binding is a digest, not a second plaintext copy of the import.
        assert!(!serde_json::to_string(&saved).unwrap().contains("selected"));
    }
    #[test]
    fn disconnect_and_identical_reconnect_supersede_in_flight_renewal() {
        assert!(selection_matches(1, 1, Some("original"), "original"));
        assert!(!selection_matches(2, 1, None, "original"));
        assert!(!selection_matches(3, 1, Some("original"), "original"));
        assert!(!selection_matches(
            1,
            1,
            Some("another-account"),
            "original"
        ));
    }
    #[test]
    fn encrypted_renewal_keeps_selection_stamp_and_rejects_late_writes() {
        let root = tempfile::tempdir().unwrap();
        let manual_path = root.path().join("manual_cookies.json");
        let cache_path = root.path().join("geminiapps_session.json");
        let mut cookies = ManualCookies::default();
        cookies.set("geminiapps", "selected-private-header");
        cookies.set("claude", "unrelated-private-header");
        crate::secure_file::write_string(&manual_path, &serde_json::to_string(&cookies).unwrap())
            .unwrap();
        let stamp = std::fs::metadata(&manual_path).unwrap().modified().unwrap();
        let original = std::fs::read(&manual_path).unwrap();
        save_at(
            &cache_path,
            &cookies,
            1,
            "selected-private-header",
            1,
            "renewed-private-header",
        )
        .unwrap();
        assert_eq!(
            read_at(&cache_path).unwrap().cookie_header,
            "renewed-private-header"
        );
        assert_eq!(
            std::fs::metadata(&manual_path).unwrap().modified().unwrap(),
            stamp
        );
        assert_eq!(std::fs::read(&manual_path).unwrap(), original);
        #[cfg(windows)]
        assert!(
            !std::fs::read_to_string(&cache_path)
                .unwrap()
                .contains("renewed-private-header")
        );
        cookies.remove("geminiapps");
        save_at(
            &cache_path,
            &cookies,
            2,
            "selected-private-header",
            1,
            "late-disconnected",
        )
        .unwrap();
        cookies.set("geminiapps", "selected-private-header");
        save_at(
            &cache_path,
            &cookies,
            3,
            "selected-private-header",
            1,
            "late-reconnected",
        )
        .unwrap();
        assert_eq!(
            read_at(&cache_path).unwrap().cookie_header,
            "renewed-private-header"
        );
    }
}
