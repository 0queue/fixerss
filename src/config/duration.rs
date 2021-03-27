#[derive(serde_derive::Deserialize, Debug)]
#[serde(untagged)]
pub enum DurationConfig {
    Days { days: u32 },
    Hours { hours: u32 },
}

impl Into<chrono::Duration> for DurationConfig {
    fn into(self) -> chrono::Duration {
        match self {
            DurationConfig::Days { days } => chrono::Duration::days(days as i64),
            DurationConfig::Hours { hours } => chrono::Duration::hours(hours as i64)
        }
    }
}