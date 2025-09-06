use crate::config::Config;
use crate::logger::*;
use crate::notifier;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use tray_item::TrayItem;

pub fn start_tray<F>(booster_enabled: Arc<Mutex<bool>>, get_load: F)
where
    F: Fn() -> f64 + Send + 'static,
{
    let tray =
        TrayItem::new("Speed Booster", "assets/green.png").expect("Failed to create tray icon");

    let booster_clone = Arc::clone(&booster_enabled);
    let tray = Arc::new(Mutex::new(tray));

    {
        let tray_clone = Arc::clone(&tray);
        booster_clone.lock().unwrap(); // ensure Arc clone
    }

    // Pause
    {
        let booster_clone = Arc::clone(&booster_enabled);
        let tray_clone = Arc::clone(&tray);
        tray.lock()
            .unwrap()
            .add_menu_item("Pause Booster", move || {
                let mut enabled = booster_clone.lock().unwrap();
                *enabled = false;
                log("Booster paused from tray");
                notifier::notify_with_icon_throttle(
                    "Speed Booster",
                    "Booster paused by user",
                    "assets/yellow.png",
                );
            })
            .expect("Failed to add menu item");
    }

    // Resume
    {
        let booster_clone = Arc::clone(&booster_enabled);
        let tray_clone = Arc::clone(&tray);
        tray.lock()
            .unwrap()
            .add_menu_item("Resume Booster", move || {
                let mut enabled = booster_clone.lock().unwrap();
                *enabled = true;
                log("Booster resumed from tray");
                notifier::notify_with_icon_throttle(
                    "Speed Booster",
                    "Booster resumed by user",
                    "assets/green.png",
                );
            })
            .expect("Failed to add menu item");
    }

    // Quit
    tray.lock()
        .unwrap()
        .add_menu_item("Quit", || {
            log("Tray quit clicked");
            std::process::exit(0);
        })
        .expect("Failed to add quit menu item");

    // Background thread: dynamic icon update
    let booster_clone = Arc::clone(&booster_enabled);
    let tray_clone = Arc::clone(&tray);
    thread::spawn(move || {
        loop {
            let enabled = *booster_clone.lock().unwrap();
            let load = get_load();
            let icon_path = if !enabled {
                "assets/yellow.png"
            } else if load < 0.2 {
                "assets/green.png"
            } else {
                "assets/orange.png"
            };

            if let Ok(mut t) = tray_clone.lock() {
                let _ = t.set_icon_from_file(icon_path);
            }

            thread::sleep(Duration::from_secs(5));
        }
    });

    tray.lock()
        .unwrap()
        .wait_for_message()
        .expect("Tray message loop failed");
}
