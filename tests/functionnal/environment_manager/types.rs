use std::{env::var, time::Duration};

use assert_fs::TempDir;
// Takes the SLEEP_TIME env variable or default to 2sec
use lazy_static::lazy_static;
use wormhole::network::ip::IpP;

lazy_static! {
    pub static ref SLEEP_TIME: Duration =
        Duration::from_secs_f32(if let Ok(str_st) = var("SLEEP_TIME") {
            str_st.parse().unwrap_or(2.0)
        } else {
            2.0
        });
}

pub const SERVICE_MAX_PORT: u16 = 9999;
pub const SERVICE_MIN_PORT: u16 = 8081;
pub const SERVICE_BIN: &str = "./target/debug/wormholed";
pub const CLI_BIN: &str = "./target/debug/wormhole";

pub struct Service {
    pub instance: std::process::Child,
    pub ip: IpP,
    pub pods: Vec<(String, IpP, TempDir)>, // (network_name, ip, dir)
}

impl Drop for Service {
    fn drop(&mut self) {
        let exit_status = self.instance.wait();
        std::thread::sleep(*SLEEP_TIME);

        match &exit_status {
            Ok(status) => log::info!(
                "Stopped service {}\nExitStatus: {:?}\nStopped pods:\n{:?}",
                self.ip,
                status,
                self.pods
                    .iter()
                    .map(|(_, ip, _)| ip.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Err(e) => log::error!(
                "Error when stopping service {}\nExit error: {:?}\n^ This service pods:\n{:?}",
                self.ip,
                e,
                self.pods
                    .iter()
                    .map(|(_, ip, _)| ip.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        }
        assert!(exit_status.is_ok())
    }
}

pub enum StopMethod {
    CtrlD,
    CliStop,
    Kill,
}
