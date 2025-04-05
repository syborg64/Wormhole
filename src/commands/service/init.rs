use std::{env, path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    commands::cli_commands::PodArgs,
    config::{types::Config, GlobalConfig, LocalConfig},
    network::server::Server,
    pods::{declarations::Pod, whpath::WhPath},
};

//TODO - check if msg[1] is really a mount_point
pub async fn init(
    pod_args: PodArgs,
) -> Result<(GlobalConfig, LocalConfig, Arc<Server>, WhPath), String> {
    let global_config: GlobalConfig =
        GlobalConfig::read(".global_config.toml").map_err(|e| e.to_string())?;
    let local_config: LocalConfig =
        LocalConfig::read(".local_config.toml").map_err(|e| e.to_string())?;
    let server: Arc<Server> = Arc::new(Server::setup(&local_config.general.address).await);
    let mount_point = pod_args.path.unwrap_or(".".into());
    Ok((global_config, local_config, server, mount_point))
}
