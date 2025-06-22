use std::{env::home_dir, fs};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    dev_block: String,
    launcher: String,
    lounge: Option<String>,
    mac: String,
}

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Serialization(toml::de::Error),
    InvalidHome
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) ->Self {
        Error::IO(e)
    }
}
impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::Serialization(e)
    }
}
impl Config {
    pub fn from_file(cfg_file: &str) -> Result<Config, Error> {
       
        let mut home_path = home_dir()
            .ok_or(Error::InvalidHome)?;
        
        home_path.push(&cfg_file);

        let path = fs::read_to_string(&home_path)?;
        let conf = toml::from_str(&path)?;

        Ok(conf)
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
