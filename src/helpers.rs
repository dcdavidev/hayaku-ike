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
        }
        if line.starts_with("SwapFree:") {
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
