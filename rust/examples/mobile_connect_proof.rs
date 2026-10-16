//! Disposable emulator fixture. Contains only public synthetic data and keys.
use codexbar::mobile_bridge::*;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let phase_path = std::env::args().nth(1).expect("phase file path");
    let pairing = Pairing {
        companion_id: "11111111-1111-4111-8111-111111111111".into(),
        shared_key_base64_url: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
    };
    let source = Arc::new(move |pairing: &Pairing| {
        let second = std::fs::read_to_string(&phase_path).is_ok_and(|value| value.trim() == "1");
        let now = chrono::Utc::now().timestamp();
        let providers = ["codex", "claude", "copilot"]
            .into_iter()
            .map(|id| {
                let scope = account_scope(id, "synthetic-worker-proof");
                MobileProvider {
                    id: id.into(),
                    updated_at_epoch_seconds: now,
                    status: "ready".into(),
                    subject_id: account_subject(pairing, &scope),
                    windows: vec![MobileWindow {
                        label: "Session".into(),
                        used_fraction: if second { 0.1 } else { 0.8 },
                        resets_at_epoch_seconds: Some(now + if second { 18000 } else { 3 }),
                        window_duration_seconds: Some(18000),
                    }],
                    budget: None,
                    balance: None,
                    local_usage: None,
                    reset_credits: (id == "codex").then_some(MobileResetCredits {
                        available_count: if second { 3 } else { 1 },
                        expires_at_epoch_seconds: vec![],
                    }),
                    history_scope_id: account_subject(pairing, &scope),
                    history: vec![MobileQuotaHistory {
                        label: "Session".into(),
                        samples: vec![
                            MobileHistoryPoint {
                                t: now - 600,
                                u: 0.6,
                                r: None,
                            },
                            MobileHistoryPoint {
                                t: now,
                                u: if second { 0.1 } else { 0.8 },
                                r: None,
                            },
                        ],
                    }],
                }
            })
            .collect();
        MobileSnapshot {
            schema_version: 1,
            source: SOURCE.into(),
            generated_at_epoch_seconds: now,
            providers,
            spend: Some(MobileSpend {
                updated_at_epoch_seconds: now,
                reporting_period: "rolling:30".into(),
                time_zone: "UTC".into(),
                cost_tracking_enabled: true,
                rows: vec![MobileSpendRow {
                    provider_id: "codex".into(),
                    currency: "USD".into(),
                    refreshing: false,
                    seven_day: Some(20.0),
                    thirty_day: Some(80.0),
                    period_cost: Some(80.0),
                    seven_day_tokens: Some(100000),
                    thirty_day_tokens: Some(400000),
                    period_tokens: Some(400000),
                    tokens_lower_bound: false,
                    daily: vec![MobileSpendDay {
                        day: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                        tokens: Some(5000),
                        cost_usd: Some(0.5),
                        is_scanned: true,
                    }],
                }],
            }),
        }
    });
    let server = MobileServer::start("127.0.0.1:43829".parse().unwrap(), pairing, source)
        .await
        .unwrap();
    println!("Synthetic Mobile Connect fixture ready on loopback:43829");
    tokio::time::sleep(std::time::Duration::from_secs(600)).await;
    server.stop().await;
}
