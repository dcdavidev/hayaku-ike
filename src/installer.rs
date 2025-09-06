use std::fs;
use std::path::Path;
use std::process::Command;

pub fn install_service() {
    let src_path = Path::new("assets/speedup.service");
    let dst_path = Path::new("/etc/systemd/system/speedup.service");

    if !src_path.exists() {
        eprintln!("❌ Service template not found at {:?}", src_path);
        return;
    }

    fs::copy(src_path, dst_path).expect("Failed to copy service file. Run as root!");

    // Reload systemd
    Command::new("systemctl")
        .args(&["daemon-reload"])
        .status()
        .expect("Failed to reload systemd");

    // Enable & start service
    Command::new("systemctl")
        .args(&["enable", "--now", "speedup.service"])
        .status()
        .expect("Failed to enable/start service");

    println!("✅ Service installed and started!");
}
