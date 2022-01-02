#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
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

impl TryFrom<ServerConfig> for std::net::SocketAddr {
    type Error = std::net::AddrParseError;

    fn try_from(value: ServerConfig) -> Result<Self, Self::Error> {
        format!("{}:{}", value.address, value.port).parse()
    }
}