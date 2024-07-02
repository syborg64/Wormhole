use serde::Deserialize;

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
