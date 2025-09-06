use colored::*;
use std::{fs, process::Command, thread, time::Duration};

/// Run one booster cycle (CPU, swap, cache)
pub fn run_boost_cycle() {
    println!("{}", "🚀 Running booster cycle...".bold().green());

    // CPU governor
    if command_exists("cpufreq-set") {
        run_sudo("cpufreq-set", &["-r", "-g", "performance"]);
    }

    // Swappiness
    if command_exists("sysctl") {
        run_sudo("sysctl", &["vm.swappiness=10"]);
    }

    // Swap
    if command_exists("swapoff") && command_exists("swapon") {
        if let Ok(swap_used) = get_swap_usage() {
            if swap_used > 0 {
                run_sudo("swapoff", &["-a"]);
                run_sudo("swapon", &["-a"]);
            }
        }
    }

    // Page cache
    if fs::metadata("/proc/sys/vm/drop_caches").is_ok() {
        run_sudo("sh", &["-c", "echo 3 > /proc/sys/vm/drop_caches"]);
    }

    println!("{}", "✅ Boost cycle completed.".bold().green());
}

/// Start the daemon loop (interval in seconds)
pub fn start_daemon(interval: u64) {
    loop {
        run_boost_cycle();
        println!("{}", format!("Sleeping {}s...\n", interval).bold().green());
        thread::sleep(Duration::from_secs(interval));
    }
}

// ---- Helpers ----

fn command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", cmd))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_sudo(cmd: &str, args: &[&str]) {
    let _ = Command::new("sudo").arg(cmd).args(args).status();
}

/// Returns used swap in KiB
fn get_swap_usage() -> Result<u64, std::io::Error> {
    let data = fs::read_to_string("/proc/meminfo")?;
    let mut total = 0;
    let mut free = 0;
    for line in data.lines() {
        if line.starts_with("SwapTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            total = parts[1].parse::<u64>().unwrap_or(0);
        }
        if line.starts_with("SwapFree:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            free = parts[1].parse::<u64>().unwrap_or(0);
        }
    }
    Ok(total.saturating_sub(free))
}
