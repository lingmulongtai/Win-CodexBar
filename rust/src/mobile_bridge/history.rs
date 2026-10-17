//! Canonical, account/connection-bound PC history; no merging by provider alone.
use super::{MobileHistoryPoint, MobileQuotaHistory, MobileWindow};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

const RETENTION: i64 = 90 * 86400;
// Keep a time-based archive. Repeatedly halving a 128-point ring makes almost
// every older day disappear at a normal refresh cadence.
const BUCKET_SECONDS: i64 = 10 * 60;
const MAX_SAMPLES: usize = 2 * 90 * 24 * 6 + 2; // Two samples per ten-minute bucket, for 90 days.
const MAX_STORE_BYTES: u64 = 32 * 1024 * 1024;
// DPAPI ciphertext is base64-encoded inside a JSON envelope on Windows.
const MAX_PLAINTEXT_BYTES: u64 = MAX_STORE_BYTES / 4 * 3 - 4096;

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
            let point = MobileHistoryPoint {
                t: at,
                u: window.used_fraction,
                r: window.resets_at_epoch_seconds,
            };
            // Retain the first observation and the latest real sample in each
            // ten-minute bucket; no interpolated or fabricated observations.
            if samples.len() > 1
                && samples[samples.len() - 2..].iter().all(|sample| {
                    sample.t / BUCKET_SECONDS == at / BUCKET_SECONDS
                        && sample.r == point.r
                        && sample.u <= point.u
                })
            {
                *samples.last_mut().unwrap() = point;
            } else {
                samples.push(point);
            }
            if samples.len() > MAX_SAMPLES {
                compact_daily(samples);
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

impl HistoryStore {
    fn checkpoint(&mut self) -> Option<String> {
        let mut raw = serde_json::to_string(self).ok()?;
        if raw.len() as u64 > MAX_STORE_BYTES / 2 {
            // Coarsen older data rather than write a file the next launch rejects.
            for accounts in self.providers.values_mut() {
                for account in accounts {
                    for samples in account.windows.values_mut() {
                        compact_daily(samples);
                    }
                }
            }
            raw = serde_json::to_string(self).ok()?;
        }
        (raw.len() as u64 <= MAX_PLAINTEXT_BYTES).then_some(raw)
    }
}

fn compact_daily(samples: &mut Vec<MobileHistoryPoint>) {
    let recent = samples.len().saturating_sub(32);
    let mut retained: Vec<MobileHistoryPoint> = Vec::new();
    for (index, sample) in samples.iter().enumerate() {
        if index > 1
            && index < recent
            && retained
                .last()
                .is_some_and(|last| last.t / 86400 == sample.t / 86400)
        {
            *retained.last_mut().unwrap() = sample.clone();
        } else {
            retained.push(sample.clone());
        }
    }
    *samples = retained;
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
                    std::fs::metadata(path).is_ok_and(|meta| meta.len() <= MAX_STORE_BYTES)
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
        && let Some(raw) = store.checkpoint()
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

    #[test]
    fn thirty_days_remain_available_after_restart_and_more_refreshes() {
        let start = 1_700_000_000;
        let mut store = HistoryStore::default();
        for minute in (0..45 * 1440).step_by(5) {
            store.record("codex", [1; 32], start + minute * 60, &window(0.4));
        }
        let raw = serde_json::to_string(&store).unwrap();
        let mut restarted: HistoryStore = serde_json::from_str(&raw).unwrap();
        let now = start + 45 * 86400;
        restarted.record("codex", [1; 32], now, &window(0.5));
        let projected = restarted.project("codex", &[1; 32], now);
        let samples = &projected[0].samples;
        assert!(samples[0].t <= now - 29 * 86400);
        assert_eq!(samples.last().unwrap().t, now);
        assert!(samples.len() <= 64);
        for day in 1..30 {
            assert!(samples.iter().any(|sample| (now - sample.t) / 86400 == day));
        }
        assert!(restarted.project("codex", &[2; 32], now).is_empty());
    }

    #[test]
    fn encrypted_atomic_checkpoint_restores_history() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("history.json");
        let mut store = HistoryStore::default();
        store.record("geminiapps", [3; 32], 1_700_000_000, &window(0.2));
        crate::secure_file::write_string(&path, &serde_json::to_string(&store).unwrap()).unwrap();
        let restored: HistoryStore =
            serde_json::from_str(&crate::secure_file::read_string(&path).unwrap()).unwrap();
        assert_eq!(
            restored.project("geminiapps", &[3; 32], 1_700_000_001)[0].samples[0].u,
            0.2
        );
    }

    #[test]
    fn full_account_capacity_checkpoint_fits_restart_read_limit() {
        let now = 1_800_000_000;
        let mut store = HistoryStore::default();
        let samples: Vec<_> = (0..600)
            .map(|index| MobileHistoryPoint {
                t: now - (599 - index) * 3 * 3600,
                u: 0.12345678901234568,
                r: Some(now + 3600),
            })
            .collect();
        for provider in 0..32 {
            let accounts = (0..8)
                .map(|account| ProviderHistory {
                    scope: [account; 32],
                    windows: (0..8)
                        .map(|window| (format!("{window}{}", "x".repeat(240)), samples.clone()))
                        .collect(),
                })
                .collect();
            store.providers.insert(provider.to_string(), accounts);
        }
        let raw = store.checkpoint().unwrap();
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("full-history.json");
        crate::secure_file::write_string(&path, &raw).unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() <= MAX_STORE_BYTES);
        let restored: HistoryStore =
            serde_json::from_str(&crate::secure_file::read_string(&path).unwrap()).unwrap();
        let history = restored.project("0", &[0; 32], now);
        assert_eq!(history.len(), 8);
        assert!(history[0].samples[0].t <= now - 29 * 86400);
    }

    #[test]
    fn reset_boundaries_are_not_overwritten_inside_a_bucket() {
        let at = 1_700_000_400;
        let mut store = HistoryStore::default();
        store.record("codex", [1; 32], at, &window(0.4));
        store.record("codex", [1; 32], at + 1, &window(0.8));
        let mut reset = window(0.0);
        reset[0].resets_at_epoch_seconds = Some(1_800_100_000);
        store.record("codex", [1; 32], at + 2, &reset);
        reset[0].used_fraction = 0.1;
        store.record("codex", [1; 32], at + 3, &reset);
        let history = store.project("codex", &[1; 32], at + 4);
        assert_eq!(history[0].samples.len(), 4);
        assert_eq!(history[0].samples[1].u, 0.8);
        assert_eq!(history[0].samples[2].u, 0.0);
    }
}
