use crate::cli::{Args, Commands};
use crate::config::Config;
use clap::Parser;
use env_logger;
use log::{error, info, warn};
use nix::unistd::{self, Pid};
use std::process;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

mod booster;
mod cli;
mod config;
mod notifier;

fn main() {
    env_logger::init();
    info!("🚀 Starting Hayaku-Ike...");

    let args = Args::parse();
    let config = Config::load("/etc/hayaku-ike.d/config.toml");

    match args.command {
        Commands::Daemon => {
            info!("Running in daemon mode...");
            let is_paused = Arc::new(AtomicBool::new(false));
            booster::run_daemon_boost(&config, is_paused);
        }
        Commands::Boost => {
            info!("Running single-shot boost...");
            booster::run_single_shot_boost(&config);
        }
        Commands::Pause { pid } => {
            info!("Attempting to pause daemon with PID: {}", pid);
            if !unistd::Uid::effective().is_root() {
                warn!("Insufficient privileges to send signal. Please run with sudo.");
                return;
            }
            let nix_pid = Pid::from_raw(pid as i32);
            match nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGUSR1) {
                Ok(_) => info!("✅ Signal sent successfully. Daemon pause toggled."),
                Err(e) => {
                    error!("❌ Failed to send signal to PID {}: {}", pid, e);
                    process::exit(1);
                }
            }
        }
    }
}
