use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    commands::cli_commands::PodConf,
    config::{GlobalConfig, LocalConfig},
    error::{CliError, CliResult},
};

pub async fn apply(
    local_config: Arc<RwLock<LocalConfig>>,
    global_config: Arc<RwLock<GlobalConfig>>,
    args: PodConf,
) -> Result<(), CliError> {
    
    Ok(())
}
