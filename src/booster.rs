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
            println!("{}", "⏸ Booster paused by user".yellow());
            log("Booster paused by user");
            notifier::notify_with_icon_throttle(
                "Speed Booster",
                "Booster paused by user",
                &config.icons.paused,
            );
            run_boost_cycle(Some(false), &config);
            idle_cycles = 0;
        } else if load < config.idle_load_threshold {
            idle_cycles += 1;
            println!("{}", "💤 System idle detected, running booster".green());
            log("System idle detected, running booster");
            if idle_cycles >= config.min_idle_cycles_for_notify
                || (last_load - load).abs() > config.load_change_threshold
            {
                notifier::notify_with_icon_throttle(
                    "Speed Booster",
                    "System idle detected, running booster",
                    &config.icons.active,
                );
            }
            run_boost_cycle(Some(true), &config);
        } else {
            let msg = format!("⚡ System busy (load {:.2}), skipping booster", load);
            println!("{}", msg.yellow());
            log(&msg);
            if (last_load - load).abs() > config.load_change_threshold {
                notifier::notify_with_icon_throttle("Speed Booster", &msg, &config.icons.busy);
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
    }

    // CPU governor
    if command_exists("cpufreq-set") && enabled {
        if let Ok(cores) = get_cpu_cores() {
            let msg = format!("Detected {} CPU cores, setting performance governor", cores);
            println!("{}", msg);
            log(&msg);
            run_sudo("cpufreq-set", &["-r", "-g", "performance"]);
        }
    }

    if command_exists("sysctl") && enabled {
        run_sudo("sysctl", &["vm.swappiness=10"]);
        log("Swappiness set to 10");
    }

    if command_exists("swapoff") && command_exists("swapon") {
        if let Ok(swap_used) = get_swap_usage() {
            if enabled && swap_used > 0 {
                let msg = format!("Used swap: {} KiB. Refreshing swap...", swap_used);
                println!("{}", msg);
                log(&msg);
                run_sudo("swapoff", &["-a"]);
                run_sudo("swapon", &["-a"]);
            }
        }
    }

    if fs::metadata("/proc/sys/vm/drop_caches").is_ok() && enabled {
        println!("{}", "Dropping page cache...".green());
        log("Dropping page cache");
        run_sudo("sh", &["-c", "echo 3 > /proc/sys/vm/drop_caches"]);
    }

    if enabled {
        println!("{}", "✅ Booster cycle completed.".bold().green());
        log("Booster cycle completed");
        notifier::notify_with_icon_throttle(
            "Speed Booster",
            "Booster cycle completed",
            &config.icons.active,
        );
    }
}
