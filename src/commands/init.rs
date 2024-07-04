// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::error::Error;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BasicConfig {
    pub name: String,
}

#[must_use]
pub fn init(path: &std::path::PathBuf, name: &str) -> Result<(), Box<dyn Error>> {
    fs::read_dir(path).map(|_| ())?;
    fs::create_dir_all((&path).join(".wormhole"))?;
    let config = BasicConfig { name: name.to_owned() };
    let serialized = toml::to_string(&config)?;
    fs::write((&path).join(".wormhole/config.toml"), serialized)?;
    crate::commands::register(path, name)?;
    Ok(())
}
