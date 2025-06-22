use log::{LevelFilter, error, info};
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use std::os::fd::AsFd;
use std::process::Command;
use std::{io};
use systemd_journal_logger::JournalLog;
pub mod config;
use config::Config;

const CFG_FILE: &str = ".config/hyprps/config";

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

    let cfg = match Config::from_file(CFG_FILE) {
        Ok(cfg) =>cfg,
        Err(_e) => {
            error!("Something in the config is wrong. Perhaps you should check it.");
            std::process::exit(1)
        }
    };

    if !cfg.validate() {
        error!("Some values in the config are wrong. Perhaps you should fill them in.");
        std::process::exit(1)
    }
    info!("Configuration file validated and ready to go! Hyprps is monitoring device {}-{} and {} will start in lounge mode {}.", 
        cfg.get_device(), cfg.get_mac(), cfg.get_launcher(),cfg.get_lounge().get_or_insert("NO")
    );

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
                            info!("{} We detected you pressed a button on your controller, didn’t you? Sorry, someone is playing instead of you.", cfg.get_device());
                            ensure_launcher_running(&cfg);
                        }
                    }
                } else if action == "remove" {
                    if let Some(node) = node {
                        if node.as_ref() == cfg.get_device() {
                            info!("{} Time to do some other stuff.", cfg.get_device());
                        }
                    }
                }
            }
        }
        info!("Hyprps has shut down. What a shame.");
    }
}
