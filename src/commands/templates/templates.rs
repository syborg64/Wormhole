// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::error::Error;
use std::fs;

use super::{general_config, pod_config};

#[must_use]
pub fn templates(path: &std::path::PathBuf, name: &str) -> Result<(), Box<dyn Error>> {
    fs::read_dir(path).map(|_| ())?;
    fs::create_dir_all((&path).join(".wormhole"))?;
    general_config(path, name)?;
    pod_config(path)?;
    crate::commands::register(path, name)?;
    Ok(())
}
