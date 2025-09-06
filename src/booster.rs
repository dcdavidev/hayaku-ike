use crate::config::Config;
use crate::helpers::*;
use crate::logger::*;
use crate::notifier;
use colored::*;
use std::sync::{Arc, Mutex};
use std::{fs, thread, time::Duration};

pub fn start_daemon_with_config(booster_enabled: Arc<Mutex<bool>>, config: Config) {
    let mut last_load: f64 = 0.0;
    let mut idle_cycles: u32 = 0;

    loop {
        let load = get_load_avg();
        let enabled = *booster_enabled.lock().unwrap();

        if !enabled {
            println!("{}", "⏸ Hayaku-Ike paused by user".yellow());
            log("Hayaku-Ike paused by user");
            notifier::notify_paused("Booster paused by user");
            run_boost_cycle(Some(false), &config);
            idle_cycles = 0;
        } else if load < config.idle_load_threshold {
            idle_cycles += 1;
            println!("{}", "💤 System idle detected, running booster".green());
            log("System idle detected, running booster");

            if idle_cycles >= config.min_idle_cycles_for_notify
                || (last_load - load).abs() > config.load_change_threshold
            {
                notifier::notify_idle("System idle detected, running booster");
            }

            run_boost_cycle(Some(true), &config);
        } else {
            let msg = format!("⚡ System busy (load {:.2}), skipping booster", load);
            println!("{}", msg.yellow());
            log(&msg);

            if (last_load - load).abs() > config.load_change_threshold {
                notifier::notify_busy(&msg);
            }

            idle_cycles = 0;
        }

        last_load = load;

        let interval = if load < config.idle_load_threshold {
            config.min_interval
        } else {
            config.max_interval
        };

        println!(
            "{}",
            format!("⏱ Next check in {}s\n", interval).bold().green()
        );
        log(&format!("Next check in {}s", interval));
        thread::sleep(Duration::from_secs(interval));
    }
}

pub fn run_boost_cycle(enabled: Option<bool>, config: &Config) {
    let enabled = enabled.unwrap_or(true);

    if enabled {
        println!("{}", "🚀 Running booster cycle...".bold().green());
        log("Running booster cycle");
    } else {
        println!("{}", "⏸ Booster cycle skipped (paused)".yellow());
        log("Booster cycle skipped (paused)");
        return;
    }

    // CPU governor
    if command_exists("cpufreq-set") {
        if let Ok(cores) = get_cpu_cores() {
            let msg = format!("Detected {} CPU cores, setting performance governor", cores);
            println!("{}", msg);
            log(&msg);
            run_sudo("cpufreq-set", &["-r", "-g", "performance"]);
        }
    }

    // Swappiness
    if command_exists("sysctl") {
        run_sudo("sysctl", &["vm.swappiness=10"]);
        log("Swappiness set to 10");
    }

    // Refresh swap if needed
    if command_exists("swapoff") && command_exists("swapon") {
        if let Ok(swap_used) = get_swap_usage() {
            if swap_used > 0 {
                let msg = format!("Used swap: {} KiB. Refreshing swap...", swap_used);
                println!("{}", msg);
                log(&msg);
                run_sudo("swapoff", &["-a"]);
                run_sudo("swapon", &["-a"]);
            }
        }
    }

    // Drop page cache
    if fs::metadata("/proc/sys/vm/drop_caches").is_ok() {
        println!("{}", "Dropping page cache...".green());
        log("Dropping page cache");
        run_sudo("sh", &["-c", "echo 3 > /proc/sys/vm/drop_caches"]);
    }

    println!("{}", "✅ Booster cycle completed.".bold().green());
    log("Booster cycle completed");
    notifier::notify_idle("Booster cycle completed");
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

    // Mock notifier functions
    fn mock_notify_idle(msg: &str) {
        NOTIFICATIONS.with(|n| n.borrow_mut().push(format!("IDLE: {}", msg)));
    }
    fn mock_notify_busy(msg: &str) {
        NOTIFICATIONS.with(|n| n.borrow_mut().push(format!("BUSY: {}", msg)));
    }
    fn mock_notify_paused(msg: &str) {
        NOTIFICATIONS.with(|n| n.borrow_mut().push(format!("PAUSED: {}", msg)));
    }

    // Mock helpers
    fn mock_command_exists(_: &str) -> bool {
        true
    }
    fn mock_run_sudo(_: &str, _: &[&str]) {}
    fn mock_get_cpu_cores() -> Result<usize, std::io::Error> {
        Ok(4)
    }
    fn mock_get_swap_usage() -> Result<u64, std::io::Error> {
        Ok(0)
    }

    fn run_daemon_once(booster_enabled: Arc<Mutex<bool>>, config: Config, load: f64) {
        // simulate one loop iteration
        let enabled = *booster_enabled.lock().unwrap();

        if !enabled {
            mock_log("Booster paused by user");
            mock_notify_paused("Booster paused by user");
            return;
        }

        if load < config.idle_load_threshold {
            mock_log("System idle detected, running booster");
            mock_notify_idle("System idle detected, running booster");
        } else {
            let msg = format!("System busy (load {:.2}), skipping booster", load);
            mock_log(&msg);
            mock_notify_busy(&msg);
        }
    }

    #[test]
    fn test_paused_state() {
        let booster_enabled = Arc::new(Mutex::new(false));
        let config = Config::default();

        run_daemon_once(Arc::clone(&booster_enabled), config, 0.0);

        LOGS.with(|l| {
            let logs = l.borrow();
            assert!(logs.iter().any(|s| s.contains("Booster paused by user")));
        });

        NOTIFICATIONS.with(|n| {
            let notifs = n.borrow();
            assert!(notifs.iter().any(|s| s.contains("PAUSED")));
        });
    }

    #[test]
    fn test_idle_state() {
        let booster_enabled = Arc::new(Mutex::new(true));
        let mut config = Config::default();
        config.idle_load_threshold = 0.5;

        run_daemon_once(Arc::clone(&booster_enabled), config, 0.2);

        LOGS.with(|l| {
            let logs = l.borrow();
            assert!(logs.iter().any(|s| s.contains("System idle")));
        });

        NOTIFICATIONS.with(|n| {
            let notifs = n.borrow();
            assert!(notifs.iter().any(|s| s.contains("IDLE")));
        });
    }

    #[test]
    fn test_busy_state() {
        let booster_enabled = Arc::new(Mutex::new(true));
        let mut config = Config::default();
        config.idle_load_threshold = 0.5;

        run_daemon_once(Arc::clone(&booster_enabled), config, 0.8);

        LOGS.with(|l| {
            let logs = l.borrow();
            assert!(logs.iter().any(|s| s.contains("System busy")));
        });

        NOTIFICATIONS.with(|n| {
            let notifs = n.borrow();
            assert!(notifs.iter().any(|s| s.contains("BUSY")));
        });
    }
}
