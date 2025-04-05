use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct PodBasicConfig {
    storage: Storage,
    strategy: Strategy,
}

#[derive(Serialize, Deserialize)]
struct Storage {
    pub max_disk_space: i8,
}

impl Default for Storage {
    fn default() -> Self {
        Self { max_disk_space: 95 }
    }
}

#[derive(Serialize, Deserialize)]
struct Strategy {
    pub redundancy_priority: u64,
    pub cache: u8,
}

impl Default for Strategy {
    fn default() -> Self {
        Self {
            redundancy_priority: 0,
            cache: 2,
        }
    }
}

pub fn pod_config(path: &std::path::PathBuf) -> Result<(), Box<dyn Error>> {
    let config = PodBasicConfig {
        storage: Storage::default(),
        strategy: Strategy::default(),
    };
    let serialized = toml::to_string(&config)?;
    fs::write((&path).join(".wormhole/pod_config.toml"), serialized)?;
    Ok(())
}
