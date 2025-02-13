use std::fs::File;
use std::io::Read;

#[must_use]
pub fn parse_toml_file<T>(path_to_config_file: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    let mut file = File::open(path_to_config_file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let config: T = toml::from_str(&content)?;
    log::info!("{config:?}");
    Ok(config)
}
