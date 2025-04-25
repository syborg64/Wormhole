use crate::{commands::{cli_commands::RestoreConf, PodCommand}, error::{CliError, CliResult, CliSuccess}};
use tokio::sync::mpsc;

pub async fn restore(tx: mpsc::UnboundedSender<PodCommand>, args: RestoreConf) -> CliResult {
    Ok(CliSuccess::Message("bread".to_owned()))
}