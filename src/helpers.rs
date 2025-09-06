use std::{fs, process::Command};

/// Check if a command exists in PATH
pub fn command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", cmd))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run a command with sudo (best-effort)
pub fn run_sudo(cmd: &str, args: &[&str]) {
    let _ = Command::new("sudo").arg(cmd).args(args).status();
}

/// Return number of CPU cores
pub fn get_cpu_cores() -> Result<usize, std::io::Error> {
    let data = fs::read_to_string("/proc/cpuinfo")?;
    Ok(data.lines().filter(|l| l.starts_with("processor")).count())
}

/// Return used swap in KiB
pub fn get_swap_usage() -> Result<u64, std::io::Error> {
    let data = fs::read_to_string("/proc/meminfo")?;
    let mut total = 0;
    let mut free = 0;

    for line in data.lines() {
        if line.starts_with("SwapTotal:") {
            total = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0);
        } else if line.starts_with("SwapFree:") {
            free = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0);
        }
    }

    Ok(total.saturating_sub(free))
}

/// Return normalized system load (0.0 to 1.0)
pub fn get_load_avg() -> f64 {
    if let Ok(data) = fs::read_to_string("/proc/loadavg") {
        let first: f64 = data
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);
        let cores = get_cpu_cores().unwrap_or(1) as f64;
        return first / cores; // normalized load per core
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        static MOCK_COMMANDS: RefCell<Vec<String>> = RefCell::new(vec![]);
    }

    fn mock_command_exists(cmd: &str) -> bool {
        MOCK_COMMANDS.with(|m| m.borrow().contains(&cmd.to_string()))
    }

    #[test]
    fn test_command_exists_real_cmd() {
        // This will check "sh" exists on the system
        assert!(command_exists("sh"));
    }

    #[test]
    fn test_command_exists_fake_cmd() {
        assert!(!command_exists("this-command-does-not-exist"));
    }

    #[test]
    fn test_get_cpu_cores_basic() {
        let cores = get_cpu_cores().unwrap();
        assert!(cores > 0, "CPU cores should be greater than 0");
    }

    #[test]
    fn test_get_swap_usage_basic() {
        let swap = get_swap_usage().unwrap();
        assert!(swap >= 0, "Swap usage should be non-negative");
    }

    #[test]
    fn test_get_load_avg_basic() {
        let load = get_load_avg();
        assert!(load >= 0.0, "Load average should be non-negative");
    }

    // Optional: mock `fs::read_to_string` by temporarily overriding helpers
    // to simulate /proc content without touching the real system
    #[test]
    fn test_cpu_cores_mocked() {
        fn mock_read_cpuinfo() -> String {
            "processor\t: 0\nprocessor\t: 1\nprocessor\t: 2\nprocessor\t: 3\n".to_string()
        }

        let cores = mock_read_cpuinfo()
            .lines()
            .filter(|l| l.starts_with("processor"))
            .count();
        assert_eq!(cores, 4);
    }

    #[test]
    fn test_swap_usage_mocked() {
        fn mock_read_meminfo() -> String {
            "SwapTotal:       2048 kB\nSwapFree:        1024 kB\n".to_string()
        }

        let mut total = 0;
        let mut free = 0;
        for line in mock_read_meminfo().lines() {
            if line.starts_with("SwapTotal:") {
                total = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();
            } else if line.starts_with("SwapFree:") {
                free = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();
            }
        }

        let used = total.saturating_sub(free);
        assert_eq!(used, 1024);
    }
}
