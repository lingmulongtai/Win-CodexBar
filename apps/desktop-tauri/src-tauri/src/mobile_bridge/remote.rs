//! Small opt-in allowlist; remote callers never receive the full Settings document.
use super::*;
use codexbar::cost_reporting_period::CostReportingPeriod;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Update {
    expected_revision: String,
    refresh_interval_secs: Option<u64>,
    reporting_period: Option<String>,
}

pub(crate) fn settings_revision(settings: &Settings) -> String {
    mobile_bridge::account_scope(
        "mobile-settings-v2",
        &format!(
            "{}\n{}\n{}",
            settings.refresh_interval_secs,
            settings.adaptive_refresh,
            settings.spend_dashboard_period
        ),
    )
    .iter()
    .map(|byte| format!("{byte:02x}"))
    .collect()
}

fn settings_view(allowed: bool) -> serde_json::Value {
    let settings = Settings::load();
    serde_json::json!({"allowed":allowed,"revision":settings_revision(&settings),
        "refreshIntervalSecs":settings.refresh_interval_secs,"adaptiveRefresh":settings.adaptive_refresh,
        "reportingPeriod":settings.spend_dashboard_period.to_string()})
}

fn patch(update: &Update) -> Result<crate::commands::SettingsUpdate, &'static str> {
    if update.expected_revision.len() != 64
        || (update.refresh_interval_secs.is_none() && update.reporting_period.is_none())
    {
        return Err("invalid");
    }
    if update
        .refresh_interval_secs
        .is_some_and(|value| value != 0 && !(60..=86400).contains(&value))
    {
        return Err("invalid");
    }
    let period = update
        .reporting_period
        .as_ref()
        .map(|raw| {
            CostReportingPeriod::parse(raw)
                .filter(|value| value.to_string() == *raw)
                .map(|value| value.to_string())
                .ok_or("invalid")
        })
        .transpose()?;
    Ok(crate::commands::SettingsUpdate {
        refresh_interval_secs: update.refresh_interval_secs,
        // Choosing a fixed interval explicitly turns Adaptive off, as General does.
        adaptive_refresh: update.refresh_interval_secs.map(|_| false),
        spend_dashboard_period: period,
        ..Default::default()
    })
}

/// The gate is invalidated before stopping/revoking a listener. Holding it through
/// save makes an in-flight update finish before revocation, never after it.
pub(super) fn source(
    app: tauri::AppHandle,
    gate: Arc<Mutex<bool>>,
    allowed: bool,
) -> mobile_bridge::ControlSource {
    Arc::new(move |resource, payload| {
        let active = gate.lock().map_err(|_| ())?;
        if !*active {
            return Err(());
        }
        let outcome = if resource == "settings.get" {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Empty {}
            if serde_json::from_slice::<Empty>(payload).is_err() {
                "invalid"
            } else {
                "ok"
            }
        } else if !allowed {
            "disabled"
        } else {
            match serde_json::from_slice::<Update>(payload)
                .ok()
                .and_then(|update| patch(&update).ok().map(|patch| (update, patch)))
            {
                None => "invalid",
                Some((update, patch)) => match crate::commands::update_settings_checked(
                    app.clone(),
                    patch,
                    Some(&update.expected_revision),
                ) {
                    Ok(_) => {
                        schedule_spend_refresh(&app);
                        "ok"
                    }
                    Err(error) if error == "mobile-settings-conflict" => "conflict",
                    Err(_) => "unavailable",
                },
            }
        };
        serde_json::to_vec(&serde_json::json!({"status":outcome,"settings":settings_view(allowed)}))
            .map_err(|_| ())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remote_allowlist_rejects_credentials_unknown_fields_and_invalid_ranges() {
        for raw in [
            r#"{"expectedRevision":"x","apiKeys":{}}"#,
            r#"{"expectedRevision":"x","address":"0.0.0.0"}"#,
        ] {
            assert!(serde_json::from_str::<Update>(raw).is_err());
        }
        let mut update = Update {
            expected_revision: "a".repeat(64),
            refresh_interval_secs: Some(60),
            reporting_period: Some("rolling:30".into()),
        };
        assert!(patch(&update).is_ok());
        update.refresh_interval_secs = Some(1);
        assert!(patch(&update).is_err());
        update.refresh_interval_secs = None;
        update.reporting_period = Some("rolling:0".into());
        assert!(patch(&update).is_err());
    }
    #[test]
    fn local_and_other_device_changes_invalidate_the_revision() {
        let a = Settings::default();
        let revision = settings_revision(&a);
        let mut b = a.clone();
        b.refresh_interval_secs += 60;
        assert_ne!(settings_revision(&b), revision);
        b = a.clone();
        b.spend_dashboard_period = CostReportingPeriod::Rolling(7);
        assert_ne!(settings_revision(&b), revision);
        b = a.clone();
        b.adaptive_refresh = !a.adaptive_refresh;
        assert_ne!(settings_revision(&b), revision);
    }
}
