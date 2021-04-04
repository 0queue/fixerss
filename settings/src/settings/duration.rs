#[derive(serde_derive::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DurationSettings {
    Days { days: u32 },
    Hours { hours: u32 },
}

impl From<DurationSettings> for chrono::Duration {
    fn from(duration_settings: DurationSettings) -> Self {
        match duration_settings {
            DurationSettings::Days { days } => chrono::Duration::days(days as i64),
            DurationSettings::Hours { hours } => chrono::Duration::hours(hours as i64),
        }
    }
}
