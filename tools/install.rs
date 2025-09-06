use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, exit};
use std::thread;
use std::time::Duration;

/// ANSI colors
const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";

fn log_info(msg: &str) {
    println!("{}[INFO]{} {}", BLUE, RESET, msg);
}
fn log_warn(msg: &str) {
    println!("{}[WARN]{} {}", YELLOW, RESET, msg);
}
fn log_error(msg: &str) {
    eprintln!("{}[ERROR]{} {}", RED, RESET, msg);
}
fn log_success(msg: &str) {
    println!("{}[OK]{} {}", GREEN, RESET, msg);
}

fn detect_distro() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("ID=") {
                return line[3..].trim_matches('"').to_lowercase();
            }
        }
    }
    log_error("Could not detect Linux distro");
    exit(1);
}

fn detect_arch() -> String {
    if let Ok(output) = Command::new("uname").arg("-m").output() {
        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
    "unknown".to_string()
}

fn is_root() -> bool {
    env::var("USER").map(|u| u == "root").unwrap_or(false)
}

fn run(cmd: &str, args: &[&str], use_sudo: bool) {
    let status = if use_sudo && !is_root() {
        log_info(&format!("sudo {} {}", cmd, args.join(" ")));
        Command::new("sudo").arg(cmd).args(args).status()
    } else {
        log_info(&format!("{} {}", cmd, args.join(" ")));
        Command::new(cmd).args(args).status()
    }
    .expect("Failed to run command");

    if !status.success() {
        log_error(&format!("Command failed: {} {:?}", cmd, args));
        exit(1);
    }
}

fn retry<F: Fn() -> bool>(mut f: F, retries: u8, wait_sec: u64) {
    for i in 0..=retries {
        if f() {
            return;
        }
        if i < retries {
            thread::sleep(Duration::from_secs(wait_sec));
        }
    }
    log_error(&format!("Operation failed after {} retries", retries));
    exit(1);
}

fn install_packages(distro: &str, arch: &str) {
    let mut pkgs = match distro {
        "ubuntu" | "debian" => vec![
            "pkg-config",
            "libglib2.0-dev",
            "libgtk-3-dev",
            "cpufrequtils",
            "procps",
            "libappindicator3-dev",
        ],
        "fedora" | "centos" | "rhel" => vec![
            "pkgconf-pkg-config",
            "glib2-devel",
            "gtk3-devel",
            "cpupowerutils",
            "procps-ng",
            "libappindicator-gtk3",
        ],
        "arch" | "manjaro" => vec![
            "pkgconf",
            "glib2",
            "gtk3",
            "cpupower",
            "procps-ng",
            "libappindicator-gtk3",
        ],
        "opensuse-tumbleweed" | "opensuse-leap" => vec![
            "pkg-config",
            "glib2-devel",
            "gtk3-devel",
            "cpupower",
            "procps",
            "libappindicator3-devel",
        ],
        "alpine" => vec![
            "pkgconfig",
            "glib-dev",
            "gtk+3.0-dev",
            "cpupower",
            "procps",
            "libappindicator-gtk3-dev",
        ],
        _ => {
            log_error(&format!("Unsupported distro: {}", distro));
            exit(1);
        }
    };

    if arch.starts_with("arm") || arch == "aarch64" {
        log_info(&format!("ARM architecture detected: {}", arch));
        pkgs.push("libappindicator3-1:arm");
    }

    for pkg in pkgs {
        log_info(&format!("Installing package: {}", pkg));
        let args = match distro {
            "ubuntu" | "debian" => ["install", "-y", pkg],
            "fedora" | "centos" | "rhel" => ["install", "-y", pkg],
            "arch" | "manjaro" => ["-Syu", "--noconfirm", pkg],
            "opensuse-tumbleweed" | "opensuse-leap" => ["install", "-y", pkg],
            "alpine" => ["add", "--no-cache", pkg],
            _ => &[],
        };
        let manager = match distro {
            "ubuntu" | "debian" => "apt",
            "fedora" | "centos" | "rhel" => "dnf",
            "arch" | "manjaro" => "pacman",
            "opensuse-tumbleweed" | "opensuse-leap" => "zypper",
            "alpine" => "apk",
            _ => "",
        };
        retry(
            || {
                run(manager, &args, true);
                true
            },
            3,
            5,
        );
        log_success(&format!("Installed package: {}", pkg));
    }
}

fn setup_systemd_service() {
    if !is_root() {
        log_warn("Non-root user: skipping systemd service setup");
        return;
    }
    let src = Path::new("assets/service/hayaku-ike.service");
    let dst = Path::new("/etc/systemd/system/hayaku-ike.service");
    if !src.exists() {
        log_error("Service template not found");
        return;
    }

    run("cp", &[src.to_str().unwrap(), dst.to_str().unwrap()], true);
    run("systemctl", &["daemon-reload"], true);
    run(
        "systemctl",
        &["enable", "--now", "hayaku-ike.service"],
        true,
    );

    log_success("Systemd service installed and started");
}

fn build_hayaku_ike() {
    log_info("Building Hayaku-Ike...");
    run("cargo", &["build", "--release"], false);
    log_success("Hayaku-Ike built successfully");
}

fn prompt_user() -> bool {
    println!("Select installation mode:");
    println!("1) Full system install (requires sudo, sets up systemd)");
    println!("2) User-only build (no sudo, no systemd)");
    print!("Enter choice (1 or 2): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim(), "1")
}

fn main() {
    let full_install = prompt_user();
    let distro = detect_distro();
    let arch = detect_arch();
    log_info(&format!("Detected distro: {}, arch: {}", distro, arch));

    if full_install {
        install_packages(&distro, &arch);
        setup_systemd_service();
    } else {
        log_warn("User-only mode selected: skipping package installation and systemd setup");
    }

    build_hayaku_ike();
    log_success("✅ Hayaku-Ike installation complete!");
}
