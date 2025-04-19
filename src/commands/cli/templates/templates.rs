// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::fs;

use crate::commands::{default_global_config, default_local_config};
use crate::config::types::Config;
use crate::pods::whpath::WhPath;

#[must_use]
pub fn templates(path: &WhPath, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let global_config = default_global_config();
    let local_config = default_local_config(name);
    path.clone().set_absolute();
    fs::read_dir(path.inner.clone()).map(|_| ())?;
    local_config.write(path.join(".local_config.toml").inner)?;
    global_config.write(path.join(".global_config.toml").inner)?;
    Ok(())
}
