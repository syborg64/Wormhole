use std::sync::Arc;

use log::error;
use tokio::sync::mpsc;

use crate::{
    commands::{cli_commands::PodArgs, PodCommand},
    config::{GlobalConfig, LocalConfig},
    network::server::Server,
    pods::{declarations::Pod, whpath::WhPath},
};

use super::read_config;

//TODO - check if msg[1] is really a mount_point
pub async fn init(
    tx: mpsc::UnboundedSender<PodCommand>,
    pod_args: PodArgs,
) -> Result<String, String> {
    match pod_value(&pod_args).await {
        Ok((global_config, local_config, server, mount_point)) => {
            let new_pod = match Pod::new(
                pod_args.name,
                mount_point,
                global_config.general.peers,
                server.clone(),
                local_config.general.address,
            )
            .await
            {
                Ok(pod) => pod,
                Err(e) => return Err(format!("Pod creation error: {}", e)),
            };

            match tx.send(PodCommand::AddPod(new_pod)) {
                Ok(_) => Ok("Pod added successfully".to_string()),
                Err(e) => Err(format!("PodCommand send error: {}", e)),
            }
        }
        Err(e) => {
            error!("Initialization error: {}", e);
            Err(format!("Initialization error: {}", e))
        }
    }
}
async fn pod_value(
    pod_args: &PodArgs,
) -> Result<(GlobalConfig, LocalConfig, Arc<Server>, WhPath), String> {
    let (global_config, local_config, server) = read_config().await?;
    let mount_point = pod_args.path.clone().unwrap_or(".".into());
    Ok((global_config, local_config, server, mount_point))
}
