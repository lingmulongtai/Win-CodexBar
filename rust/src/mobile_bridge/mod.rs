//! Opt-in encrypted snapshots for the Android companion. Provider credentials
//! and raw provider responses are deliberately absent from this wire model.

mod protocol;
mod server;

pub use protocol::{Pairing, account_scope, account_subject, is_private_address};
use serde::{Deserialize, Serialize};
pub use server::{MobileServer, SnapshotSource};

pub const DEFAULT_PORT: u16 = 43825;
pub const SOURCE: &str = "win-codexbar";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileSnapshot {
    pub schema_version: u8,
    pub source: String,
    pub generated_at_epoch_seconds: i64,
    pub providers: Vec<MobileProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileProvider {
    pub id: String,
    pub updated_at_epoch_seconds: i64,
    pub status: String,
    pub subject_id: Option<String>,
    pub windows: Vec<MobileWindow>,
    pub budget: Option<MobileBudget>,
    pub balance: Option<MobileBalance>,
    pub reset_credits: Option<MobileResetCredits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileWindow {
    pub label: String,
    pub used_fraction: f64,
    pub resets_at_epoch_seconds: Option<i64>,
    pub window_duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileBudget {
    pub used: f64,
    pub limit: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileBalance {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileResetCredits {
    pub available_count: u32,
    pub expires_at_epoch_seconds: Vec<i64>,
}
