mod booster;
mod installer;

fn main() {
    if std::env::args().any(|a| a == "--install") {
        installer::install_service();
        return;
    }

    println!("🚀 Starting Speed Booster Daemon...");
    booster::start_daemon(60); // fully functional now
}
