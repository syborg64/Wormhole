use std::{fs, path::Path, str};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

/** NOTE
 * To add elements in the configuration file :
 * To create a superior field like [field], create a new structure and add it to the Metadata struct
 * Minors fields are named in the structure you added to Metadata
 * the section name is the same as the name of the value of your new struct in Metadata
 */

pub trait Config: Serialize + DeserializeOwned {
    fn write<S: AsRef<Path>>(&self, path: S) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = toml::to_string(self)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    fn read<S: AsRef<Path>>(path: S) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
}

impl<T: Serialize + DeserializeOwned> Config for T {}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LocalConfig {
    pub general: GeneralLocalConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GeneralLocalConfig {
    pub name: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GlobalConfig {
    pub general: GeneralGlobalConfig,
    pub redundancy: RedundancyConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GeneralGlobalConfig {
    pub peers: Vec<String>,
    pub ignore_paths: Vec<String>,
    pub pods_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RedundancyConfig {
    pub number: u64,
}

//OLD
//OLD
//OLD
//OLD

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Network {
    pub name: String,
    pub peers: Vec<String>,
}

impl Network {
    pub fn new(peers: Vec<String>, name: String) -> Self {
        Self { name, peers }
    }
}
