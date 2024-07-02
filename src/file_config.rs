use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use toml::{from_str, Value};

/** NOTE
 * To add elements in the configuration file :
 * To create a superior field like [field], create a new structure and add it to the Metadata struct
 * Minors fields are named in the structure you added to Metadata
 * the section name is the same as the name of the value of your new struct in Metadata
 */

#[derive(Debug, Deserialize)]
pub struct Metadata {
    essential: EssentialConfig,
    optional: Option<OptionalConfig>,
}

#[derive(Debug, Deserialize)]
pub struct EssentialConfig {
    name: String,
    ip: String,
}

#[derive(Debug, Deserialize)]
pub struct OptionalConfig {
    redundancy: Option<bool>,
}


//TODO - change error feedback to find out which section or value is missing
pub fn parse_toml_file(path_to_config_file: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    let mut file = File::open(path_to_config_file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let metadata: Metadata = toml::from_str(&content)?;
    println!("{:?}", metadata);
    Ok(metadata)
}
