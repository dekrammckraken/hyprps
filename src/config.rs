use std::{env::home_dir, fs};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    dev_block: String,
    launcher: String,
    lounge: Option<String>,
    mac: String,
}

impl Config {
    pub fn from_file(cfg_file: &str) -> Config {
        let mut home_path = home_dir().ok_or("failed").unwrap();
        home_path.push(&cfg_file);

        let config_string = fs::read_to_string(&home_path).expect("INVALID_CONFIG_FILE");

        let conf = toml::from_str(&config_string);

        conf.unwrap()
    }

    pub fn get_device(&self) -> &str {
        &self.dev_block
    }
    pub fn get_launcher(&self) -> &str {
        &self.launcher
    }
    pub fn get_mac(&self) -> &str {
        &self.mac
    }
     pub fn get_lounge(&self) -> Option<&str> {
        self.lounge.as_deref()
    }
    pub fn validate(&self) -> bool {
        
        if self.dev_block.is_empty(){
            return false;
        }

        if self.mac.is_empty() {
            return false;
        }

        if self.launcher.is_empty() {
            return false;
        }
        true
    }
}
