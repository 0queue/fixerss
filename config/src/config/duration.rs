#[derive(serde_derive::Deserialize, Debug)]
#[serde(untagged)]
pub enum DurationConfig {
    Days { days: u32 },
    Hours { hours: u32 },
}

impl From<DurationConfig> for chrono::Duration {
    fn from(duration_config: DurationConfig) -> Self {
        match duration_config {
            DurationConfig::Days { days } => chrono::Duration::days(days as i64),
            DurationConfig::Hours { hours } => chrono::Duration::hours(hours as i64),
        }
    }
}
