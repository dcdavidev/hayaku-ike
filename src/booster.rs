use crate::config::Config;
use crate::notifier;
use log::{error, info, warn};
use nix::unistd::Uid;
use signal_hook::{consts::signal::*, iterator::Signals};
use std::io::{self, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{thread, time::Duration};
use sysinfo::System;

fn is_root() -> bool {
    if Uid::effective().is_root() {
        true
    } else {
        warn!("Run as root for full functionality.");
        false
    }
}

fn apply_performance_boost() -> io::Result<()> {
    info!("Setting CPU governor to performance...");
    let cores = System::new().cpus().len();
    for i in 0..cores {
        let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", i);
        let mut file = std::fs::File::create(&path)?;
        file.write_all(b"performance")?;
    }
    Ok(())
}

fn restore_performance_boost() -> io::Result<()> {
    info!("Restoring CPU governor to powersave...");
    let cores = System::new().cpus().len();
    for i in 0..cores {
        let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", i);
        let mut file = std::fs::File::create(&path)?;
        file.write_all(b"powersave")?;
    }
    Ok(())
}

fn get_system_metrics() {
    let mut system = System::new();
    system.refresh_all();
    let cpu_load = system.global_cpu_info().cpu_usage() / (system.cpus().len() as f32);
    let memory_usage_mb = system.used_memory() as f64 / 1024.0 / 1024.0;
    let swap_usage_mb = system.used_swap() as f64 / 1024.0 / 1024.0;
    info!(
        "Metrics: CPU {:.2}%, Mem {:.2}MB, Swap {:.2}MB",
        cpu_load, memory_usage_mb, swap_usage_mb
    );
}

pub fn run_single_shot_boost(config: &Config) {
    info!("Single-shot boost...");
    notifier::notify_started(config);
    if !is_root() {
        return;
    }

    get_system_metrics();

    let system = System::new();
    let cpu_load = system.global_cpu_info().cpu_usage() / (system.cpus().len() as f32);

    if cpu_load < config.idle_load_threshold as f32 {
        if let Err(e) = apply_performance_boost() {
            error!("Boost failed: {}", e);
        } else {
            notifier::notify_boost_applied(config);
        }
    } else {
        notifier::notify_busy(config, "System too busy.");
    }

    if let Err(e) = restore_performance_boost() {
        error!("Restore failed: {}", e);
    }
    notifier::notify_stopped(config);
}

pub fn run_daemon_boost(config: &Config, is_paused: Arc<AtomicBool>) {
    info!("Daemon started");
    notifier::notify_started(config);
    if !is_root() {
        return;
    }

    let running = Arc::new(AtomicBool::new(true));

    // Pause/resume handler
    let is_paused_clone = Arc::clone(&is_paused);
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGUSR1]).unwrap();
        for _ in signals.forever() {
            let current_state = is_paused_clone.load(Ordering::Relaxed);
            is_paused_clone.store(!current_state, Ordering::Relaxed);
            info!("Paused toggled: {}", !current_state);
        }
    });

    // Shutdown handler
    let running_clone = Arc::clone(&running);
    let config_clone = config.clone();
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
        for _ in signals.forever() {
            info!("Shutdown signal received");
            let _ = restore_performance_boost();
            notifier::notify_stopped(&config_clone);
            running_clone.store(false, Ordering::Relaxed);
            break;
        }
    });

    while running.load(Ordering::Relaxed) {
        if is_paused.load(Ordering::Relaxed) {
            notifier::notify_paused(config, "Paused");
            thread::sleep(Duration::from_secs(config.max_interval));
            continue;
        }

        get_system_metrics();

        let system = System::new();
        let cpu_load = system.global_cpu_info().cpu_usage() / (system.cpus().len() as f32);

        if cpu_load < config.idle_load_threshold as f32 {
            let _ = apply_performance_boost();
            notifier::notify_boost_applied(config);
        } else {
            let _ = restore_performance_boost();
            notifier::notify_boost_restored(config);
        }

        thread::sleep(Duration::from_secs(config.min_interval));
    }

    info!("Daemon stopped");
}
