use std::fs;

use crate::{commands::cli_commands::RestoreConf, config::{types::Config, LocalConfig}, error::{CliResult, CliSuccess}, pods::{arbo::{Arbo, GLOBAL_CONFIG_FNAME, LOCAL_CONFIG_FNAME, LOCAL_CONFIG_INO}, pod::Pod}};

pub async fn restore(pod: &Pod, args: RestoreConf) -> CliResult {
    // for file in args.files.clone() {
    //     match file.as_str() {
    //         LOCAL_CONFIG_FNAME => {
    //             let global_config_path = Arbo::read_lock(&pod.fs_interface, "fs_interface::send_filesystem")?
    //                                         .get_path_from_inode_id(LOCAL_CONFIG_INO)?.set_relative();
    //             log::info!("reading Local config at {global_config_path}");
    //             let global_config_bytes = &pod.disk.read_file_to_end(global_config_path).expect("lmao l'incompétence");
    //             fs::write(pod.get_mount_point().inner, pod.)
    //         },
    //         GLOBAL_CONFIG_FNAME => {},
    //         _ => Err(ErrorCli()),
    //     }
    // }
    Ok(CliSuccess::Message("bread".to_owned()))
}