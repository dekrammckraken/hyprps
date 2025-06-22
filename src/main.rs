use log::{LevelFilter, error, info};
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use std::os::fd::AsFd;
use std::process::Command;
use std::{io};
use systemd_journal_logger::JournalLog;

const CFG_FILE: &str = ".config/hyprps/config";

pub mod config;

use config::Config;

fn ensure_launcher_running(cfg: &Config) {
    let running = Command::new("pgrep")
        .arg(&cfg.get_launcher())
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);

    if !running {
        
        let mut launcher_commmand = Command::new(&cfg.get_launcher());
        
        if let Some(lounge_param) = &cfg.get_lounge() {
            launcher_commmand.arg(lounge_param);
        }

        let mut launcher = launcher_commmand.spawn().expect("Failed to start launcher");
        let _ = launcher.wait().expect("Failed to wait on launcher");
        
        disconnect_device(&cfg.get_mac()).expect("Failed to disconnect device");
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

    let cfg = Config::from_file(CFG_FILE);
    
    if !cfg.validate() {
        eprintln!("{}", "Invalid configuration");
        std::process::exit(1)
    }


    info!("{} Starting hyprps monitoring", cfg.get_device());

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
                        if node.as_ref() == cfg.get_device() {
                            info!("{} added!", cfg.get_device());
                            ensure_launcher_running(&cfg);
                        }
                    }
                } else if action == "remove" {
                    if let Some(node) = node {
                        if node.as_ref() == cfg.get_device() {
                            info!("{} removed!", cfg.get_device());
                        }
                    }
                }
            }
        }
        info!("hyprps has been closed.");
    }
}
