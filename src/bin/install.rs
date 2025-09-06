use nix::unistd::Uid;
use std::path::Path;
use std::process::{Command, exit};

fn main() {
    println!("🚀 Hayaku-Ike installation & launch script");

    // Controllo privilegi
    if !Uid::effective().is_root() {
        println!("⚠️ Not running as root. Some steps may require sudo.");
    }

    // Compila il progetto in release
    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .status()
        .expect("Failed to run cargo build");
    if !status.success() {
        eprintln!("❌ Cargo build failed!");
        exit(1);
    }
    println!("✅ Build completed.");

    // Path al binario
    let binary_path = Path::new("target/release/hayaku-ike");
    if !binary_path.exists() {
        eprintln!("❌ Compiled binary not found: {}", binary_path.display());
        exit(1);
    }

    // Copia del servizio systemd se root
    let service_src = Path::new("assets/service/hayaku-ike.service");
    let service_dst = Path::new("/etc/systemd/system/hayaku-ike.service");

    if Uid::effective().is_root() {
        if !service_dst.exists() {
            let status = Command::new("cp")
                .args(&[service_src.to_str().unwrap(), service_dst.to_str().unwrap()])
                .status()
                .expect("Failed to copy service file");
            if !status.success() {
                eprintln!("❌ Failed to copy service file.");
                exit(1);
            }
        }

        Command::new("systemctl")
            .args(&["daemon-reload"])
            .status()
            .expect("Failed to reload systemd daemon");

        Command::new("systemctl")
            .args(&["enable", "--now", "hayaku-ike.service"])
            .status()
            .expect("Failed to enable/start Hayaku-Ike service");

        println!("✅ Systemd service installed and started.");
    } else {
        println!("⚠️ You are not root. Please run the daemon manually:");
        println!("{}", binary_path.display());
    }

    // Lancia il daemon immediatamente
    println!("🚀 Launching Hayaku-Ike daemon now...");
    let mut child = Command::new(binary_path)
        .arg("daemon")
        .spawn()
        .expect("Failed to start Hayaku-Ike daemon");

    println!("✅ Daemon started with PID: {}", child.id());
    println!("Use Ctrl+C to stop the daemon.");
    let _ = child.wait();
}
