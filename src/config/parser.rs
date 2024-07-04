use std::fs::File;
use std::io::Read;

use crate::config::types::Metadata;

pub fn parse_toml_file(path_to_config_file: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    let mut file = File::open(path_to_config_file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let metadata: Metadata = toml::from_str(&content)?;
    println!("{:?}", metadata);
    Ok(metadata)
}