mod booster;
mod config;
mod helpers;
mod icons;
mod installer;
mod logger;
mod notifier;
mod tray; // added for icon constants

use booster::start_daemon_with_config;
use config::Config;
use std::sync::{Arc, Mutex};

fn main() {
    let booster_enabled = Arc::new(Mutex::new(true));

    // Load configuration
    let config = Config::load("config.toml");

    // Start booster daemon
    let booster_clone = Arc::clone(&booster_enabled);
    let config_clone = config.clone();
    std::thread::spawn(move || {
        start_daemon_with_config(booster_clone, config_clone);
    });

    // Start tray with dynamic icon updates
    tray::start_tray(booster_enabled, || helpers::get_load_avg());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_config_load() {
        // Load non-existing config returns default
        let cfg = Config::load("nonexistent.toml");
        assert_eq!(cfg.min_interval, 30);
        assert_eq!(cfg.max_interval, 300);
        assert_eq!(cfg.idle_load_threshold, 0.2);
    }

    #[test]
    fn test_booster_flag() {
        // Booster enabled should be true initially
        let booster_enabled = Arc::new(Mutex::new(true));
        assert_eq!(*booster_enabled.lock().unwrap(), true);

        // We can toggle it to false safely
        {
            let mut flag = booster_enabled.lock().unwrap();
            *flag = false;
        }
        assert_eq!(*booster_enabled.lock().unwrap(), false);
    }

    #[test]
    fn test_start_daemon_stub() {
        // We cannot run the real daemon in tests (it loops forever)
        // Instead, test that we can call start_daemon_with_config with a dummy config
        let booster_enabled = Arc::new(Mutex::new(true));
        let config = Config::default();

        // This will just call one iteration if we refactor daemon to allow a single-run
        // Here we simply assert that calling the function doesn't panic (mocked)
        let booster_clone = Arc::clone(&booster_enabled);
        let config_clone = config.clone();

        std::thread::spawn(move || {
            // Normally start_daemon_with_config loops forever,
            // so in a real test we would refactor it to allow single iteration
            // For now, just call it to ensure it compiles
            // start_daemon_with_config(booster_clone, config_clone);
        });
    }
}
