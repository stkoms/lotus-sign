use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub lotus: LotusConfig,
    pub database: DatabaseConfig,
    pub wallet: Option<WalletConfig>,
}

#[derive(Debug, Deserialize)]
pub struct LotusConfig {
    pub host: String,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct WalletConfig {
    pub password: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    fn config_path() -> PathBuf {
        PathBuf::from("config.toml")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lotus: LotusConfig {
                host: "https://api.node.glif.io/rpc/v0".to_string(),
                token: None,
            },
            database: DatabaseConfig {
                path: "lotus_sign.db".to_string(),
            },
            wallet: None,
        }
    }
}

impl Config {
    pub fn get_password(&self) -> String {
        self.wallet
            .as_ref()
            .and_then(|w| w.password.clone())
            .unwrap_or_default()
    }
}
