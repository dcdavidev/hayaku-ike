use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;

const LOG_FILE: &str = "/var/log/hayaku-ike.log";

/// Log a message to file with timestamp
pub fn log(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let line = format!("[{}] {}\n", timestamp, message);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LOG_FILE) {
        let _ = file.write_all(line.as_bytes());
    }
}
