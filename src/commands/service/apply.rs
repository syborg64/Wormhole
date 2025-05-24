use std::{io, sync::Arc};

use parking_lot::RwLock;

use crate::{
    commands::cli_commands::PodConf,
    config::{types::Config, GlobalConfig, LocalConfig},
    error::{CliError}, pods::arbo::{GLOBAL_CONFIG_FNAME, LOCAL_CONFIG_FNAME},
};

pub async fn apply(
    local_config: Arc<RwLock<LocalConfig>>,
    global_config: Arc<RwLock<GlobalConfig>>,
    args: PodConf,
) -> Result<(), CliError> {
    for file in args.files.clone() {
        match file.as_str() {
            LOCAL_CONFIG_FNAME => {
                let conf = LocalConfig::read(&args.path.join(LOCAL_CONFIG_FNAME).inner).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let mut local_conf = LocalConfig::write_lock(&local_config, "apply::local_config")?;
                *local_conf = conf;
            },
            GLOBAL_CONFIG_FNAME => {
                let conf = GlobalConfig::read(&args.path.join(GLOBAL_CONFIG_FNAME).inner).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let mut global_conf = GlobalConfig::write_lock(&global_config, "apply::global_conf")?;
                *global_conf = conf;
            },
            _ => { return Err(CliError::InvalidArgument { arg: file })}
        }
    }


    Ok(())
}
