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
                active: "assets/icons/player-play.svg".to_string(),
                busy: "assets/icons/cpu.svg".to_string(),
                paused: "assets/icons/player-stop.svg".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = Config::default();
        assert_eq!(cfg.idle_load_threshold, 0.2);
        assert_eq!(cfg.min_interval, 30);
        assert_eq!(cfg.max_interval, 300);
        assert_eq!(cfg.load_change_threshold, 0.05);
        assert_eq!(cfg.min_idle_cycles_for_notify, 2);
    }

    #[test]
    fn config_icons_paths() {
        let cfg = Config::default();
        assert_eq!(cfg.icons.active, "assets/icons/player-play.svg");
        assert_eq!(cfg.icons.busy, "assets/icons/cpu.svg");
        assert_eq!(cfg.icons.paused, "assets/icons/player-stop.svg");
    }

    #[test]
    fn load_nonexistent_config_returns_default() {
        let cfg = Config::load("nonexistent.toml");
        // It should fallback to defaults
        assert_eq!(cfg.min_interval, 30);
        assert_eq!(cfg.max_interval, 300);
        assert_eq!(cfg.idle_load_threshold, 0.2);
    }

    #[test]
    fn load_invalid_toml_returns_default() {
        // Write an invalid TOML string to a temp file
        use std::io::Write;
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        write!(tmpfile, "invalid toml content").unwrap();

        let cfg = Config::load(tmpfile.path().to_str().unwrap());
        assert_eq!(cfg.min_interval, 30);
        assert_eq!(cfg.max_interval, 300);
        assert_eq!(cfg.idle_load_threshold, 0.2);
    }
}
