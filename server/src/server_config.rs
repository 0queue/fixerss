#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ServerConfig {
    pub settings_file: String,
    pub history_file: String,
    pub address: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            settings_file: "fixerss.toml".to_string(),
            history_file: ":memory:".to_string(),
            address: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

impl ServerConfig {
    pub fn from_env_or_default() -> ServerConfig {
        let default = ServerConfig::default();

        let settings_file = std::env::var("FIXERSS_SETTINGS_FILE")
            .unwrap_or(default.settings_file);

        let history_file = std::env::var("FIXERSS_HISTORY_FILE")
            .unwrap_or(default.history_file);

        let address = std::env::var("FIXERSS_ADDRESS")
            .unwrap_or(default.address);

        let port = std::env::var("FIXERSS_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default.port);

        ServerConfig {
            settings_file,
            history_file,
            address,
            port,
        }
    }
}

impl TryFrom<ServerConfig> for std::net::SocketAddr {
    type Error = std::net::AddrParseError;

    fn try_from(value: ServerConfig) -> Result<Self, Self::Error> {
        format!("{}:{}", value.address, value.port).parse()
    }
}