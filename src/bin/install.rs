use nix::unistd::Uid;
use std::path::Path;
use std::process::{Command, exit};

fn main() {
    println!("🚀 Hayaku-Ike installation script (simplified)");

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

    // Informazioni sul servizio systemd
    let service_src = Path::new("assets/service/hayaku-ike.service");
    let service_dst = Path::new("/etc/systemd/system/hayaku-ike.service");

    println!("Service file location: {}", service_src.display());
    if !Uid::effective().is_root() {
        println!("⚠️ Run the following manually as root to install the systemd service:");
        println!(
            "sudo cp {} {}",
            service_src.display(),
            service_dst.display()
        );
        println!("sudo systemctl daemon-reload");
        println!("sudo systemctl enable --now hayaku-ike.service");
    } else {
        // Copia e abilita il servizio automaticamente se root
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
    }

    println!("🎉 Hayaku-Ike installation complete!");
}
