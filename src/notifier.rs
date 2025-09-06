use lazy_static::lazy_static;
use notify_rust::Notification;
use std::path::Path;
use std::sync::Mutex;

lazy_static! {
    static ref LAST_NOTIFICATION: Mutex<String> = Mutex::new(String::new());
}

/// Use a custom icon for notification, throttled to avoid repeats
pub fn notify_with_icon_throttle(title: &str, message: &str, icon_path: &str) {
    let mut last = LAST_NOTIFICATION.lock().unwrap();

    // Only notify if message changed
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
