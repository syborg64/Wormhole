use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use toml::{from_str, Value};

#[derive(Deserialize, Debug)]
pub struct Config {
    name: String,
    ip: String,
}

pub fn recover_config(path_to_config_file: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(path_to_config_file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let config: Config = toml::from_str(&content)?;

    Ok(config)
}
