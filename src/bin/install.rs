use nix::unistd::Uid;
use std::fs;
use std::path::Path;
use std::process::Command;

fn run(cmd: &str, args: &[&str], sudo: bool) {
    let (program, final_args) = if sudo {
        let mut v = vec![cmd];
        v.extend_from_slice(args);
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
    }
}

fn main() {
    if !Uid::effective().is_root() {
        eprintln!("⚠️ Please run with sudo to install the service.");
        return;
    }

    let src = Path::new("assets/service/hayaku-ike.service");
    let dst = Path::new("/etc/systemd/system/hayaku-ike.service");

    if !dst.exists() {
        fs::copy(src, dst).expect("Failed to copy service file");
        println!("✅ Service file installed to /etc/systemd/system/");
    } else {
        println!("⚠️ Service file already exists, skipping copy");
    }

    run("systemctl", &["daemon-reload"], false);
    run(
        "systemctl",
        &["enable", "--now", "hayaku-ike.service"],
        false,
    );

    println!("✅ Hayaku-Ike installed and running!");
}
