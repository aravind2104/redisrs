use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub password: Option<String>,
    pub port: Option<u16>,
    pub snapshot_interval: Option<u64>,
    pub aof_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            password: None,
            port: Some(6379),
            snapshot_interval: Some(60),
            aof_path: Some("appendonly.aof".to_string()),
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Config {
        match fs::read_to_string(path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Failed to parse config file: {}", e);
                    Config::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to read config file: {}", e);
                Config::default()
            }
        }
    }
}
