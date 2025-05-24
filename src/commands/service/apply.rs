use std::sync::Arc;
use parking_lot::RwLock;
use crate::{
    commands::cli_commands::PodConf,
    config::{types::Config, GlobalConfig, LocalConfig},
    error::{CliError, CliSuccess}, pods::arbo::{GLOBAL_CONFIG_FNAME, LOCAL_CONFIG_FNAME},
};


//TODO - pas modifier les address ip, mettre un warning disant qu'elle ne seront pas prise en compte
pub async fn apply(
    local_config: Arc<RwLock<LocalConfig>>,
    global_config: Arc<RwLock<GlobalConfig>>,
    args: PodConf,
) -> Result<CliSuccess, CliError> {
    for file in args.files {
        match file.as_str() {
            LOCAL_CONFIG_FNAME => {
                let conf = LocalConfig::read(&args.path.join(LOCAL_CONFIG_FNAME).inner).map_err(|err| CliError::BoxError { arg: err })?;
                LocalConfig::write_lock(&local_config, "apply::local_config")?.constructor(conf)?;
            },
            GLOBAL_CONFIG_FNAME => {
                let conf = GlobalConfig::read(&args.path.join(GLOBAL_CONFIG_FNAME).inner).map_err(|err| CliError::BoxError { arg: err })?;
                GlobalConfig::write_lock(&global_config, "apply::global_conf")?.constructor(conf)?;
            },
            _ => { return Err(CliError::InvalidArgument { arg: file })}
        }
    }
    Ok(CliSuccess::Message("The new configuration is apply".to_owned()))
}
