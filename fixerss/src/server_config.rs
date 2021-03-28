use rocket::figment::value::Dict;
use rocket::figment::value::Map;

// too many things named feed-spec flying about
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Config {
    pub(crate) feeds: String
}

impl Default for Config {
    fn default() -> Self {
        Config {
            feeds: "fixerss.toml".to_string()
        }
    }
}

// not sure how to read this (and only this?) from FIXERSS_FEEDS
impl rocket::figment::Provider for Config {
    fn metadata(&self) -> rocket::figment::Metadata {
        rocket::figment::Metadata::named("fixerss")
    }

    fn data(&self) -> Result<Map<rocket::figment::Profile, Dict>, rocket::figment::error::Error> {
        rocket::figment::providers::Serialized::defaults(self).data()
    }
}