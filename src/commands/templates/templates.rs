// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::fs;

use serde::Serialize;

use crate::pods::whpath::WhPath;

use super::{default_global_config, default_local_config};

fn write_config<T: Serialize>(config: T, path: String) -> Result<(), Box<dyn std::error::Error>> {
    fs::File::create(path.clone())?;
    let serialized = toml::to_string(&config)?;
    fs::write(&path, serialized)?;
    Ok(())
}

#[must_use]
pub fn templates(path: &WhPath, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    //REVIEW - Mettre les template en dehors de la fonction pour plus de lisibilité ? dans un autre fichier ?
    let global_config = default_global_config();
    let local_config = default_local_config(name);
    path.clone().set_absolute();
    fs::read_dir(path.inner.clone()).map(|_| ())?;
    write_config(local_config, path.join((".local_config.toml", true)).inner)?;
    write_config(
        global_config,
        path.join((".global_config.toml", true)).inner,
    )?;
    Ok(())
}
