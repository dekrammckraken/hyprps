use dirs::home_dir;
use log::{LevelFilter, error, info};
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use serde::Deserialize;
use std::os::fd::AsFd;
use std::process::Command;
use std::{fs, io};
use systemd_journal_logger::JournalLog;

const CFG_FILE: &str = ".config/hyprps/config";
const INVALID_CONFIG_FILE: &str = "Invalid configuration file";
const INVALID_HOME_DIR: &str = "Cannot access to home";

#[derive(Deserialize)]
struct Config {
    dev_block: String,
    launcher: String,
    mac: String,
}
fn get_config() -> Config {
    let mut config_path = home_dir().expect(INVALID_HOME_DIR);
    config_path.push(CFG_FILE);
    let config_string = fs::read_to_string(config_path).expect(INVALID_CONFIG_FILE);
    toml::from_str(&config_string).expect(INVALID_CONFIG_FILE)
}
fn ensure_launcher_running(cfg: &Config) {
    let running = Command::new("pgrep")
        .arg(&cfg.launcher)
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);

    if !running {
        let mut launcher = Command::new(&cfg.launcher)
            .spawn()
            .expect("Failed to start launcher");

        let _ = launcher.wait().expect("Failed to wait on launcher");
        disconnect_device(&cfg.mac).expect("Failed to disconnect device");
    }
}
fn disconnect_device(mac: &str) -> io::Result<()> {
    let status = Command::new("bluetoothctl")
        .arg("disconnect")
        .arg(mac)
        .status()?;

    if status.success() {
        info!("{} device disconnected!", mac);
        Ok(())
    } else {
        error!("{} error while disconnecting device...", mac);
        Err(io::Error::other("Disconnect failed"))
    }
}
fn main() -> io::Result<()> {
    JournalLog::new().unwrap().install().unwrap();
    log::set_max_level(LevelFilter::Info);

    let cfg = get_config();

    info!("{} Starting hyprps monitoring", cfg.dev_block);

    let monitor = udev::MonitorBuilder::new()?
        .match_subsystem("input")?
        .listen()?;

    let fd = monitor.as_fd();
    let mut fds = [PollFd::new(fd, PollFlags::POLLIN)];
    let mut iter = monitor.iter();

    loop {
        poll(&mut fds, PollTimeout::NONE)?;
        if let Some(event) = iter.next() {
            if let Some(action) = event.action() {
                let dev = event.device();
                let node = dev.devnode().map(|d| d.to_string_lossy());

                if action == "add" {
                    if let Some(node) = node {
                        if node.as_ref() == cfg.dev_block {
                            info!("{} added!", cfg.dev_block);
                            ensure_launcher_running(&cfg);
                        }
                    }
                } else if action == "remove" {
                    if let Some(node) = node {
                        if node.as_ref() == cfg.dev_block {
                            info!("{} removed!", cfg.dev_block);
                        }
                    }
                }
            }
        }
        info!("hyprps has been closed.");
    }
}
