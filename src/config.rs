use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub idle_load_threshold: f64,
    pub min_interval: u64,
    pub max_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            idle_load_threshold: 0.2,
            min_interval: 30,
            max_interval: 300,
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|_| Config::default()),
            Err(_) => Config::default(),
        }
    }
}
