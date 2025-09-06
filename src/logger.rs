use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;

const LOG_FILE: &str = "/var/log/hayaku-ike.log";

/// Log a message to file with timestamp
pub fn log(message: &str) {
    log_to_file(LOG_FILE, message);
}

/// Internal function that allows specifying the file path (for tests)
fn log_to_file(file_path: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let line = format!("[{}] {}\n", timestamp, message);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(file_path) {
        let _ = file.write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    // Use a temporary file for testing
    const TEST_LOG_FILE: &str = "test.log";

    // Ensure tests run sequentially to avoid race conditions on the same file
    lazy_static::lazy_static! {
        static ref LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_log_creates_file() {
        let _guard = LOCK.lock().unwrap();
        let _ = fs::remove_file(TEST_LOG_FILE); // clean up before test

        log_to_file(TEST_LOG_FILE, "Hello world");

        assert!(fs::metadata(TEST_LOG_FILE).is_ok());
        let content = fs::read_to_string(TEST_LOG_FILE).unwrap();
        assert!(content.contains("Hello world"));

        let _ = fs::remove_file(TEST_LOG_FILE); // clean up after test
    }

    #[test]
    fn test_log_appends() {
        let _guard = LOCK.lock().unwrap();
        let _ = fs::remove_file(TEST_LOG_FILE);

        log_to_file(TEST_LOG_FILE, "Line 1");
        log_to_file(TEST_LOG_FILE, "Line 2");

        let content = fs::read_to_string(TEST_LOG_FILE).unwrap();
        assert!(content.contains("Line 1"));
        assert!(content.contains("Line 2"));

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        let _ = fs::remove_file(TEST_LOG_FILE);
    }

    #[test]
    fn test_timestamp_format() {
        let _guard = LOCK.lock().unwrap();
        let _ = fs::remove_file(TEST_LOG_FILE);

        log_to_file(TEST_LOG_FILE, "Check timestamp");
        let content = fs::read_to_string(TEST_LOG_FILE).unwrap();
        let first_line = content.lines().next().unwrap();

        // Expect line to start with [YYYY-MM-DD HH:MM:SS]
        assert!(first_line.starts_with('[') && first_line.contains("] Check timestamp"));

        let _ = fs::remove_file(TEST_LOG_FILE);
    }
}
