use lazy_static::lazy_static;
use notify_rust::Notification;
use std::path::Path;
use std::sync::Mutex;

lazy_static! {
    static ref LAST_NOTIFICATION: Mutex<String> = Mutex::new(String::new());
}

/// Internal helper: notify with icon, throttled to avoid duplicates
fn notify_with_icon_throttle(title: &str, message: &str, icon_path: &str) {
    let mut last = LAST_NOTIFICATION.lock().unwrap();

    if &*last != message {
        let icon = if Path::new(icon_path).exists() {
            icon_path
        } else {
            "dialog-information"
        };

        let _ = Notification::new()
            .summary(title)
            .body(message)
            .icon(icon)
            .show();

        *last = message.to_string();
    }
}

/// Notify system idle / booster active
pub fn notify_idle(message: &str) {
    notify_with_icon_throttle("Hayaku-Ike", message, crate::icons::IDLE);
}

/// Notify system busy / booster delayed
pub fn notify_busy(message: &str) {
    notify_with_icon_throttle("Hayaku-Ike", message, crate::icons::BUSY);
}

/// Notify booster paused by user
pub fn notify_paused(message: &str) {
    notify_with_icon_throttle("Hayaku-Ike", message, crate::icons::PAUSED);
}
