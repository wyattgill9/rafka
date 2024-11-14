use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub log: LogConfig,
    pub broker: BrokerConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String, // "debug", "info", "warn", "error"
}

#[derive(Debug, Deserialize)]
pub struct BrokerConfig {
    pub replication_factor: u8,
    pub default_topic_partitions: u8,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    #[serde(rename = "type")]
    pub storage_type: String, // "in_memory", "rocksdb", etc.
    pub path: Option<String>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl Config {
    pub fn from_yaml(file_path: &str) -> Result<Self, ConfigError> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}
