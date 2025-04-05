use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Serialize, Deserialize)]
struct BasicConfig {
    pub name: String,
    pub network: Network,
    pub redundancy: Redundancy,
}

#[derive(Serialize, Deserialize)]
struct Network {
    pub access: Access,
    pub frequency: i64,
}

#[derive(Serialize, Deserialize)]
struct Access(String);

impl Access {
    fn new(value: &str) -> Result<Self, Box<dyn Error>> {
        if value.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Value not allowed in access",
            )));
        }
        if ["open", "demand", "whitelist", "blacklist"].contains(&value) {
            Ok(Access(value.to_string()))
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Value not authorizes in access: {}", value),
            )));
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Self {
            access: Access::new("demand").unwrap(),
            frequency: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Redundancy {
    pub amount: i64,
    pub strategy: i8,
    pub min_replication_time: i64,
    pub max_replication_time: i64,
}

impl Default for Redundancy {
    fn default() -> Self {
        Self {
            amount: 0,
            strategy: 2,
            min_replication_time: 10,
            max_replication_time: 120,
        }
    }
}

pub fn general_config(path: &std::path::PathBuf, name: &str) -> Result<(), Box<dyn Error>> {
    let config = BasicConfig {
        name: name.to_owned(),
        network: Network::default(),
        redundancy: Redundancy::default(),
    };
    let serialized = toml::to_string(&config)?;
    fs::write((&path).join(".wormhole/config.toml"), serialized)?;
    Ok(())
}
