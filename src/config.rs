use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub idle_load_threshold: f64,
    pub min_interval: u64,
    pub max_interval: u64,
    pub load_change_threshold: f64,
    pub min_idle_cycles_for_notify: u32,
    pub icons: Icons,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Icons {
    pub active: String,
    pub busy: String,
    pub paused: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            idle_load_threshold: 0.2,
            min_interval: 30,
            max_interval: 300,
            load_change_threshold: 0.05,
            min_idle_cycles_for_notify: 2,
            icons: Icons {
                active: "assets/green.png".to_string(),
                busy: "assets/orange.png".to_string(),
                paused: "assets/yellow.png".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Failed to parse config.toml: {}. Using default.", e);
                Config::default()
            }),
            Err(_) => {
                eprintln!("Config file not found at {}. Using default.", path);
                Config::default()
            }
        }
    }
}
