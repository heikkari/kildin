// [general]
// database-path = "my.db"
//
// [proxy-checker-settings]
// pagination = 100
// timeout = 10
// interval = 120
// head-dest = "https://duck.com/"
// head-loc = "https://duckduckgo.com/"
//

// serde
use serde_derive::Deserialize;

// super
use super::types;

#[derive(Deserialize)]
pub struct Config {
    pub general: General,

    #[serde(rename(deserialize = "proxy-checker-settings"))]
    pub proxy_settings: ProxyCheckerSettings,
}

#[derive(Deserialize)]
pub struct General {
    #[serde(rename(deserialize = "database-path"))]
    pub database_path: String
}

#[derive(Deserialize, Clone)]
pub struct ProxyCheckerSettings {
    pub pagination: u32,
    pub timeout: u64,
    pub interval: u64,

    #[serde(rename(deserialize = "max-fails"))]
    pub max_fails: u32,

    #[serde(rename(deserialize = "head-dest"))]
    pub dest: String,
}

#[derive(Deserialize)]
pub struct HttpServer {
    pub port: u16
}

impl Config {
    pub fn from(content: &str)
        -> Result<Self, types::AnyError>
    {
        let config = toml::from_str(content)?;
        Ok(config)
    }
}