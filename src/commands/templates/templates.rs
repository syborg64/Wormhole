// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::fs;

use libc::write;
use serde::Serialize;

use crate::{
    config::{
        types::{GeneralGlobalConfig, GeneralLocalConfig, RedundancyConfig},
        GlobalConfig, LocalConfig,
    },
    pods::whpath::WhPath,
};

use super::{general_config, pod_config};

fn write_config<T: Serialize>(config: T, path: String) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = toml::to_string(&config)?;
    fs::write(&path, serialized)?;
    Ok(())
}

#[must_use]
pub fn templates(path: &WhPath, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    //REVIEW - Mettre les template en dehors de la fonction pour plus de lisibilité ? dans un autre fichier ?
    let global_config = GlobalConfig {
        general: GeneralGlobalConfig {
            peers: Vec::new(),
            ignore_paths: Vec::new(),
        },
        redundancy: Some(RedundancyConfig { number: 12 }),
    };
    let local_config = LocalConfig {
        general: GeneralLocalConfig {
            name: name.to_string(),
            address: "127.0.0.1".to_string(),
        },
    };
    //FIXME - Pas sur que soit la bonne manière de faire
    let path = path.clone().set_absolute();
    fs::read_dir(path.inner.clone()).map(|_| ())?;
    let local_path = path.join(".local_config.toml").inner;
    fs::File::create(local_path.clone())?;
    write_config(local_config, local_path)?;
    let global_path = path.join(".global_config.toml").inner;
    fs::File::create(global_path.clone())?;
    write_config(global_config, global_path)?;
    Ok(())
}
