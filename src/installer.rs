use std::path::Path;

/// Abstract over filesystem copy
pub trait FsOps {
    fn copy(&self, src: &Path, dst: &Path) -> std::io::Result<u64>;
    fn exists(&self, path: &Path) -> bool;
}

/// Abstract over command execution
pub trait CmdRunner {
    fn run(&self, cmd: &str, args: &[&str]);
}

/// Real filesystem
pub struct RealFs;
impl FsOps for RealFs {
    fn copy(&self, src: &Path, dst: &Path) -> std::io::Result<u64> {
        std::fs::copy(src, dst)
    }
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

/// Real command runner
pub struct RealCmd;
impl CmdRunner for RealCmd {
    fn run(&self, cmd: &str, args: &[&str]) {
        let _ = std::process::Command::new(cmd).args(args).status();
    }
}

/// Install systemd service
pub fn install_service(fs: &impl FsOps, runner: &impl CmdRunner) {
    let src_path = Path::new("assets/service/hayaku-ike.service");
    let dst_path = Path::new("/etc/systemd/system/hayaku-ike.service");

    if !fs.exists(src_path) {
        eprintln!("❌ Service template not found at {:?}", src_path);
        return;
    }

    fs.copy(src_path, dst_path)
        .expect("Failed to copy service file. Run as root!");
    runner.run("systemctl", &["daemon-reload"]);
    runner.run("systemctl", &["enable", "--now", "hayaku-ike.service"]);

    println!("✅ Service installed and started!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;

    // Mock filesystem
    struct MockFs {
        pub copies: RefCell<Vec<(PathBuf, PathBuf)>>,
        pub exists: bool,
    }
    impl FsOps for MockFs {
        fn copy(&self, src: &Path, dst: &Path) -> std::io::Result<u64> {
            self.copies
                .borrow_mut()
                .push((src.to_path_buf(), dst.to_path_buf()));
            Ok(0)
        }
        fn exists(&self, _: &Path) -> bool {
            self.exists
        }
    }

    // Mock command runner
    struct MockCmd {
        pub cmds: RefCell<Vec<Vec<String>>>,
    }
    impl CmdRunner for MockCmd {
        fn run(&self, cmd: &str, args: &[&str]) {
            self.cmds.borrow_mut().push(
                std::iter::once(cmd.to_string())
                    .chain(args.iter().map(|s| s.to_string()))
                    .collect(),
            );
        }
    }

    #[test]
    fn test_service_missing() {
        let fs = MockFs {
            copies: RefCell::new(vec![]),
            exists: false,
        };
        let runner = MockCmd {
            cmds: RefCell::new(vec![]),
        };

        install_service(&fs, &runner);

        // No copy or command should have run
        assert!(fs.copies.borrow().is_empty());
        assert!(runner.cmds.borrow().is_empty());
    }

    #[test]
    fn test_service_installed() {
        let fs = MockFs {
            copies: RefCell::new(vec![]),
            exists: true,
        };
        let runner = MockCmd {
            cmds: RefCell::new(vec![]),
        };

        install_service(&fs, &runner);

        // Check that the file was copied
        let copies = fs.copies.borrow();
        assert_eq!(copies.len(), 1);
        assert_eq!(
            copies[0].0,
            PathBuf::from("assets/service/hayaku-ike.service")
        );
        assert_eq!(
            copies[0].1,
            PathBuf::from("/etc/systemd/system/hayaku-ike.service")
        );

        // Check that commands were run
        let cmds = runner.cmds.borrow();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], vec!["systemctl", "daemon-reload"]);
        assert_eq!(
            cmds[1],
            vec!["systemctl", "enable", "--now", "hayaku-ike.service"]
        );
    }
}
