use crate::config::Config;
use log::error;
use notify_rust::Notification;

/// Funzione interna per inviare notifiche desktop
fn notify(message: &str) {
    if let Err(e) = Notification::new()
        .summary("Hayaku-Ike Daemon")
        .body(message)
        .show()
    {
        error!("Failed to send notification: {}", e);
    }
}

pub fn notify_started(_config: &Config) {
    notify("🚀 Daemon started");
}

pub fn notify_stopped(_config: &Config) {
    notify("🛑 Daemon stopped");
}

pub fn notify_boost_applied(_config: &Config) {
    notify("⚡ Performance boost applied");
}

pub fn notify_boost_restored(_config: &Config) {
    notify("✅ Performance restored");
}

pub fn notify_paused(_config: &Config, message: &str) {
    notify(&format!("⏸ {}", message));
}

pub fn notify_busy(_config: &Config, message: &str) {
    notify(&format!("⚡ {}", message));
}
