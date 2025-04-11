use crate::config::{types::GeneralLocalConfig, LocalConfig};

pub fn default_local_config(name: &str) -> LocalConfig {
    return LocalConfig {
        general: GeneralLocalConfig {
            name: name.to_string(),
            address: "127.0.0.1:8080".to_string(),
        },
    };
}
