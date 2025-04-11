use std::sync::Arc;

use crate::{
    config::{types::Config, GlobalConfig, LocalConfig},
    network::server::Server,
};

pub async fn read_config() -> Result<(GlobalConfig, LocalConfig, Arc<Server>), String> {
    let global_config: GlobalConfig =
        GlobalConfig::read(".global_config.toml").map_err(|e| e.to_string())?;
    let local_config: LocalConfig =
        LocalConfig::read(".local_config.toml").map_err(|e| e.to_string())?;
    let server: Arc<Server> = Arc::new(Server::setup(&local_config.general.address).await);
    Ok((global_config, local_config, server))
}
