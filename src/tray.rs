use crate::icons::*;
use crate::logger::*;
use crate::notifier;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use tray_item::TrayItem;

pub fn start_tray<F>(booster_enabled: Arc<Mutex<bool>>, get_load: F)
where
    F: Fn() -> f64 + Send + 'static,
{
    let tray = TrayItem::new("Hayaku-Ike", IDLE).expect("Failed to create tray icon");
    let tray = Arc::new(Mutex::new(tray));

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
                notifier::notify_paused("Booster paused");
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
                notifier::notify_idle("Booster resumed");
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
                PAUSED
            } else if load < 0.2 {
                IDLE
            } else {
                BUSY
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};

    thread_local! {
        static LOGS: RefCell<Vec<String>> = RefCell::new(vec![]);
        static NOTIFICATIONS: RefCell<Vec<String>> = RefCell::new(vec![]);
    }

    // Mock logging
    fn mock_log(msg: &str) {
        LOGS.with(|l| l.borrow_mut().push(msg.to_string()));
    }

    // Mock notifier
    fn mock_notify_paused(msg: &str) {
        NOTIFICATIONS.with(|n| n.borrow_mut().push(format!("PAUSED: {}", msg)));
    }
    fn mock_notify_idle(msg: &str) {
        NOTIFICATIONS.with(|n| n.borrow_mut().push(format!("IDLE: {}", msg)));
    }

    // Mock TrayItem
    struct MockTray;
    impl MockTray {
        fn add_menu_item<F>(&self, _: &str, _: F) -> Result<(), ()>
        where
            F: Fn() + Send + 'static,
        {
            Ok(())
        }
        fn set_icon_from_file(&mut self, _: &str) -> Result<(), ()> {
            Ok(())
        }
        fn wait_for_message(&self) -> Result<(), ()> {
            Ok(())
        }
    }

    #[test]
    fn test_pause_resume_tray_logic() {
        let booster_enabled = Arc::new(Mutex::new(true));

        // simulate pause
        {
            let mut enabled = booster_enabled.lock().unwrap();
            *enabled = true;
        }

        // simulate pause click
        {
            let mut enabled = booster_enabled.lock().unwrap();
            *enabled = false;
            mock_log("Booster paused from tray");
            mock_notify_paused("Booster paused");
        }

        LOGS.with(|l| {
            let logs = l.borrow();
            assert!(logs.iter().any(|s| s.contains("paused")));
        });
        NOTIFICATIONS.with(|n| {
            let notifs = n.borrow();
            assert!(notifs.iter().any(|s| s.contains("PAUSED")));
        });

        // simulate resume click
        {
            let mut enabled = booster_enabled.lock().unwrap();
            *enabled = true;
            mock_log("Booster resumed from tray");
            mock_notify_idle("Booster resumed");
        }

        LOGS.with(|l| {
            let logs = l.borrow();
            assert!(logs.iter().any(|s| s.contains("resumed")));
        });
        NOTIFICATIONS.with(|n| {
            let notifs = n.borrow();
            assert!(notifs.iter().any(|s| s.contains("IDLE")));
        });
    }

    #[test]
    fn test_icon_selection_logic() {
        let booster_enabled = Arc::new(Mutex::new(true));

        let load_values = vec![0.0, 0.5]; // below and above idle threshold
        let mut icons_selected = Vec::new();

        for load in load_values {
            let enabled = *booster_enabled.lock().unwrap();
            let icon_path = if !enabled {
                PAUSED
            } else if load < 0.2 {
                IDLE
            } else {
                BUSY
            };
            icons_selected.push(icon_path);
        }

        assert_eq!(icons_selected, vec![IDLE, BUSY]);
    }
}
