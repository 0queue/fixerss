use rocket::figment::value::Dict;
use rocket::figment::value::Map;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ServerConfig {
    pub settings_file: String,
    pub history_file: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            settings_file: "fixerss.toml".to_string(),
            history_file: ":memory:".to_string(),
        }
    }
}

impl rocket::figment::Provider for ServerConfig {
    fn metadata(&self) -> rocket::figment::Metadata {
        rocket::figment::Metadata::named("fixerss")
    }

    fn data(&self) -> Result<Map<rocket::figment::Profile, Dict>, rocket::figment::error::Error> {
        rocket::figment::providers::Serialized::defaults(self).data()
    }
}