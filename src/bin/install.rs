use nix::unistd::Uid;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};

/// Run a command with optional sudo
fn run(cmd: &str, args: &[&str], sudo: bool) {
    let (program, final_args) = if sudo {
        let mut v = vec![cmd];
        v.extend(args);
        ("sudo", v)
    } else {
        (cmd, args.to_vec())
    };
    println!("Running: {} {:?}", program, &final_args);
    let status = Command::new(program)
        .args(&final_args)
        .status()
        .expect("Failed to run command");
    if !status.success() {
        eprintln!("Command failed: {} {:?}", cmd, args);
        exit(1);
    }
}

fn main() {
    let sudo = !Uid::effective().is_root();

    // 1️⃣ Build the project in release mode
    println!("🚀 Building Hayaku-Ike...");
    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .status()
        .expect("Cargo build failed");
    if !status.success() {
        exit(1);
    }

    // 2️⃣ Copy systemd service file if it doesn't exist
    let src = Path::new("assets/service/hayaku-ike.service");
    let dst = Path::new("/etc/systemd/system/hayaku-ike.service");
    if !dst.exists() {
        println!("📄 Installing systemd service...");
        run("cp", &[src.to_str().unwrap(), dst.to_str().unwrap()], sudo);
    } else {
        println!("ℹ️ Service already exists. Skipping copy.");
    }

    // 3️⃣ Reload systemd and enable/start the service
    run("systemctl", &["daemon-reload"], sudo);
    run(
        "systemctl",
        &["enable", "--now", "hayaku-ike.service"],
        sudo,
    );

    println!("✅ Hayaku-Ike installed and running!");
}
