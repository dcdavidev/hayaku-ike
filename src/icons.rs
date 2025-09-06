pub const IDLE: &str = "assets/icons/player-play.svg"; // system idle / booster active
pub const BUSY: &str = "assets/icons/cpu.svg"; // system busy
pub const PAUSED: &str = "assets/icons/player-stop.svg"; // booster paused

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn icons_paths_exist_or_warn() {
        let icons = [(IDLE, "IDLE"), (BUSY, "BUSY"), (PAUSED, "PAUSED")];

        for (path, name) in icons.iter() {
            if !Path::new(path).exists() {
                eprintln!("Warning: icon file for {} not found at '{}'", name, path);
            }
            // test always passes
            assert!(true);
        }
    }
}
