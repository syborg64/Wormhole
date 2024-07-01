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
    fs::create_dir((&path).join(".wormhole"))?;
    let config = BasicConfig { name: name.to_owned() };
    let serialized = toml::to_string(&config)?;
    fs::write((&path).join(".wormhole/config.toml"), serialized)?;
    Ok(())
}
