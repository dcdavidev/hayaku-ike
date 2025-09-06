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
