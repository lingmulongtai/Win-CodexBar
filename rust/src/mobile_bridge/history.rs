//! Canonical, account/connection-bound PC history; no merging by provider alone.
use super::{MobileHistoryPoint, MobileQuotaHistory, MobileWindow};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

const RETENTION: i64 = 90 * 86400;
const MAX_SAMPLES: usize = 128;

#[derive(Default, Serialize, Deserialize)]
struct HistoryStore {
    providers: BTreeMap<String, Vec<ProviderHistory>>,
}
#[derive(Serialize, Deserialize)]
struct ProviderHistory {
    scope: [u8; 32],
    windows: BTreeMap<String, Vec<MobileHistoryPoint>>,
}

impl HistoryStore {
    fn record(
        &mut self,
        provider: &str,
        scope: [u8; 32],
        at: i64,
        windows: &[MobileWindow],
    ) -> bool {
        if at <= 0 || windows.is_empty() {
            return false;
        }
        if !self.providers.contains_key(provider) && self.providers.len() >= 32 {
            return false;
        }
        let accounts = self.providers.entry(provider.to_owned()).or_default();
        accounts.retain(|entry| entry.latest() >= at - RETENTION);
        if !accounts.iter().any(|entry| entry.scope == scope) {
            if accounts.len() >= 8 {
                accounts.sort_by_key(ProviderHistory::latest);
                accounts.remove(0);
            }
            accounts.push(ProviderHistory {
                scope,
                windows: BTreeMap::new(),
            });
        }
        let entry = accounts
            .iter_mut()
            .find(|entry| entry.scope == scope)
            .unwrap();
        if entry.latest() >= at {
            return false;
        }
        let mut changed = false;
        for window in windows.iter().take(8) {
            if window.label.is_empty()
                || window.label.len() > 256
                || !window.used_fraction.is_finite()
                || !(0.0..=1.0).contains(&window.used_fraction)
            {
                continue;
            }
            let samples = entry.windows.entry(window.label.clone()).or_default();
            if samples.last().is_some_and(|sample| sample.t >= at) {
                continue;
            }
            samples.retain(|sample| sample.t >= at - RETENTION);
            samples.push(MobileHistoryPoint {
                t: at,
                u: window.used_fraction,
                r: window.resets_at_epoch_seconds,
            });
            if samples.len() > MAX_SAMPLES {
                *samples = downsample(samples, MAX_SAMPLES / 2);
            }
            changed = true;
        }
        entry
            .windows
            .retain(|label, _| windows.iter().any(|window| &window.label == label));
        changed
    }

    fn project(&self, provider: &str, scope: &[u8; 32], now: i64) -> Vec<MobileQuotaHistory> {
        let Some(entry) = self
            .providers
            .get(provider)
            .and_then(|entries| entries.iter().find(|entry| &entry.scope == scope))
        else {
            return Vec::new();
        };
        entry
            .windows
            .iter()
            .take(8)
            .map(|(label, samples)| {
                let recent: Vec<_> = samples
                    .iter()
                    .filter(|sample| sample.t >= now - 30 * 86400 && sample.t <= now + 120)
                    .cloned()
                    .collect();
                MobileQuotaHistory {
                    label: label.clone(),
                    samples: downsample(&recent, 64),
                }
            })
            .filter(|series| !series.samples.is_empty())
            .collect()
    }
}

impl ProviderHistory {
    fn latest(&self) -> i64 {
        self.windows
            .values()
            .filter_map(|samples| samples.last().map(|sample| sample.t))
            .max()
            .unwrap_or(0)
    }
}

fn downsample(samples: &[MobileHistoryPoint], max: usize) -> Vec<MobileHistoryPoint> {
    if samples.len() <= max {
        return samples.to_vec();
    }
    (0..max)
        .map(|index| samples[index * (samples.len() - 1) / (max - 1)].clone())
        .collect()
}

fn path() -> Option<std::path::PathBuf> {
    crate::settings::config_store_dir().map(|root| root.join("mobile-quota-history.json"))
}
fn store() -> &'static Mutex<HistoryStore> {
    static STORE: OnceLock<Mutex<HistoryStore>> = OnceLock::new();
    STORE.get_or_init(|| {
        Mutex::new(
            path()
                .filter(|path| {
                    std::fs::metadata(path).is_ok_and(|meta| meta.len() <= 32 * 1024 * 1024)
                })
                .and_then(|path| crate::secure_file::read_string(&path).ok())
                .and_then(|raw| serde_json::from_str(&raw).ok())
                .unwrap_or_default(),
        )
    })
}

pub fn record_history(provider: &str, scope: [u8; 32], at: i64, windows: &[MobileWindow]) {
    let Ok(mut store) = store().lock() else {
        return;
    };
    if !store.record(provider, scope, at, windows) {
        return;
    }
    if let Some(path) = path()
        && let Ok(raw) = serde_json::to_string(&*store)
        && crate::secure_file::write_string(&path, &raw).is_err()
    {
        tracing::warn!("Cannot persist Mobile Connect quota history");
    }
}

pub fn quota_history(provider: &str, scope: &[u8; 32], now: i64) -> Vec<MobileQuotaHistory> {
    store()
        .lock()
        .map(|store| store.project(provider, scope, now))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn window(used: f64) -> Vec<MobileWindow> {
        vec![MobileWindow {
            label: "Week".into(),
            used_fraction: used,
            resets_at_epoch_seconds: Some(1_800_000_000),
            window_duration_seconds: Some(604800),
        }]
    }
    #[test]
    fn account_switch_cannot_publish_another_accounts_history() {
        let mut store = HistoryStore::default();
        store.record("codex", [1; 32], 1_700_000_000, &window(0.5));
        assert!(store.project("codex", &[2; 32], 1_700_000_000).is_empty());
        store.record("codex", [2; 32], 1_700_000_001, &window(0.1));
        // Switching back (including after a restart) restores only that account's archive.
        let mut store: HistoryStore =
            serde_json::from_str(&serde_json::to_string(&store).unwrap()).unwrap();
        store.record("codex", [1; 32], 1_700_000_003, &window(0.6));
        assert_eq!(
            store.project("codex", &[1; 32], 1_700_000_004)[0]
                .samples
                .len(),
            2
        );
        assert_eq!(
            store.project("codex", &[1; 32], 1_700_000_004)[0].samples[0].u,
            0.5
        );
        assert_eq!(
            store.project("codex", &[2; 32], 1_700_000_002)[0]
                .samples
                .len(),
            1
        );
    }
    #[test]
    fn history_survives_roundtrip_and_ignores_duplicate_old_or_invalid_samples() {
        let mut store = HistoryStore::default();
        assert!(store.record("codex", [1; 32], 1_700_000_000, &window(0.5)));
        assert!(!store.record("codex", [1; 32], 1_700_000_000, &window(0.2)));
        assert!(!store.record("codex", [1; 32], 1_699_999_999, &window(0.2)));
        assert!(!store.record("codex", [1; 32], 1_700_000_001, &window(f64::NAN)));
        let loaded: HistoryStore =
            serde_json::from_str(&serde_json::to_string(&store).unwrap()).unwrap();
        assert_eq!(
            loaded.project("codex", &[1; 32], 1_700_000_002)[0].samples[0].u,
            0.5
        );
    }
    #[test]
    fn chart_bounds_keep_first_and_last_observation() {
        let samples: Vec<_> = (0..5000)
            .map(|t| MobileHistoryPoint { t, u: 0.2, r: None })
            .collect();
        let sampled = downsample(&samples, 64);
        assert_eq!(sampled.len(), 64);
        assert_eq!(sampled[0].t, 0);
        assert_eq!(sampled[63].t, 4999);
    }
}
