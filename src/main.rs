use std::fs;
use std::os::fd::{AsFd};
use std::process::Command;
use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
use serde::Deserialize;
use dirs::home_dir;

const CFG_FILE: &str = ".config/hyprps/config";
const INVALID_CONFIG_FILE: &str = "Invalid configuration file";
const INVALID_HOME_DIR: &str = "Cannot access to home";

#[derive(Deserialize)]
struct Config {
    dev_block: String,
    launcher: String,
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
        .map_or(false, |output| !output.stdout.is_empty());

    if !running {
        Command::new(&cfg.launcher)
            .spawn()
            .expect("Failed to start launcher");
        println!("Launcher started successfully");
    } else {
        println!("Launcher already started successfully");
    }
}

fn main() -> std::io::Result<()> {

    let cfg = get_config();

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
                let devnode = dev.devnode().map(|d| d.to_string_lossy());

                if action == "add" {
                    if let Some(devnode) = devnode {
                        if devnode.as_ref() == cfg.dev_block {
                            ensure_launcher_running(&cfg);
                        }
                    }
                }
                else if action == "remove" {
                    if let Some(devnode) = devnode {
                        if devnode.as_ref() == cfg.dev_block {
                            println!("Dev block removed!");
                        }
                    }
                }
            }
        }
    }
}
