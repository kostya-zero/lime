use std::fs;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: i32,
    pub pages_dir: String,
    pub static_dir: String,
    #[serde(skip)]
    pub default: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            pages_dir: "./pages".to_string(),
            static_dir: "./static".to_string(),
            default: true,
        }
    }
}

pub fn load_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path, e))?;
    let mut config = toml::from_str::<Config>(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path, e))?;
    config.default = false;
    Ok(config)
}
