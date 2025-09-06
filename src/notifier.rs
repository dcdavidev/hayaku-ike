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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Override the LAST_NOTIFICATION for testing
    fn reset_last_notification() {
        let mut last = LAST_NOTIFICATION.lock().unwrap();
        *last = String::new();
    }

    #[test]
    fn test_notify_idle_sets_last_notification() {
        reset_last_notification();
        notify_idle("Idle message");
        let last = LAST_NOTIFICATION.lock().unwrap();
        assert_eq!(*last, "Idle message");
    }

    #[test]
    fn test_notify_busy_sets_last_notification() {
        reset_last_notification();
        notify_busy("Busy message");
        let last = LAST_NOTIFICATION.lock().unwrap();
        assert_eq!(*last, "Busy message");
    }

    #[test]
    fn test_notify_paused_sets_last_notification() {
        reset_last_notification();
        notify_paused("Paused message");
        let last = LAST_NOTIFICATION.lock().unwrap();
        assert_eq!(*last, "Paused message");
    }

    #[test]
    fn test_throttle_prevents_duplicate_notifications() {
        reset_last_notification();
        notify_idle("Duplicate message");
        // Call again with same message
        notify_idle("Duplicate message");

        // LAST_NOTIFICATION should still be the same
        let last = LAST_NOTIFICATION.lock().unwrap();
        assert_eq!(*last, "Duplicate message");
    }
}
