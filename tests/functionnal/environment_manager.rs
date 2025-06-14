use std::{
    env::var,
    io,
    os::{fd::AsFd, unix::net::UnixStream},
    path::Path,
    process::ExitStatus,
    time::Duration,
};

use assert_fs::TempDir;
use lazy_static::lazy_static;
use std::process::Stdio;
use wormhole::network::ip::IpP;

use crate::functionnal::start_log;

lazy_static! {
    pub static ref SLEEP_TIME: Duration =
        Duration::from_secs_f32(if let Ok(str_st) = var("SLEEP_TIME") {
            str_st.parse().unwrap_or(2.0)
        } else {
            2.0
        });
}

const SERVICE_MAX_PORT: u16 = 9999;
const SERVICE_MIN_PORT: u16 = 8081;
const SERVICE_BIN: &str = "./target/debug/wormholed";
const CLI_BIN: &str = "./target/debug/wormhole";

pub struct Service {
    pub instance: std::process::Child,
    #[allow(dead_code)]
    #[allow(dead_code)]
    stdin: UnixStream,
    pub ip: IpP,
    pub pods: Vec<(String, IpP, TempDir)>, // (network_name, ip, dir)
}

impl Drop for Service {
    fn drop(&mut self) {
        self.instance.kill().expect("service couldn't be killed");
        // necessary for some os :
        let _ = self.instance.wait();
    }
}

pub struct EnvironmentManager {
    pub services: Vec<Service>,
}

impl EnvironmentManager {
    pub fn new() -> Self {
        start_log();
        log::trace!("SLEEP_TIME for this test is {:?}", *SLEEP_TIME);
        return EnvironmentManager {
            services: Vec::new(),
        };
    }

    fn generate_pipe(pipe_output: bool) -> Stdio {
        if pipe_output {
            Stdio::inherit()
        } else {
            Stdio::null()
        }
    }

    /// Create a service on the next available ip. No pods are created.
    pub fn add_service(&mut self, pipe_output: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut ip = self
            .services
            .iter()
            .map(|service| &service.ip)
            .max_by(|ip1, ip2| ip1.port.cmp(&ip2.port))
            .map_or_else(
                || IpP::try_from(&format!("127.0.0.1:{SERVICE_MIN_PORT}")).unwrap(),
                |ip| {
                    let mut ip = ip.clone();
                    ip.set_port(ip.port + 1);
                    ip
                },
            );

        let (write, read) = std::os::unix::net::UnixStream::pair()?;

        let instance = loop {
            assert!(
                ip.port < SERVICE_MAX_PORT,
                "service port upper limit ({SERVICE_MAX_PORT}) exceeded"
            );
            let stdio = Stdio::from(read.as_fd().try_clone_to_owned().unwrap());

            let mut instance = std::process::Command::new(SERVICE_BIN)
                .args(&[ip.to_string()])
                .stdout(Self::generate_pipe(false))
                .stderr(Self::generate_pipe(false))
                .stdin(stdio)
                .spawn()?;

            std::thread::sleep(*SLEEP_TIME);

            // checks the service viability
            let (status, out, _) = Self::cli_command(&[&ip.to_string(), "status"]).unwrap();
            if !out.contains(&ip.to_string()) {
                log::warn!(
                    "\nService created on {} isn't answering proper status.\n(code {}).\nCli stdout: \"{}\"\nWill try other ports.\n",
                    ip.to_string(),
                    status,
                    out,
                );

                instance.kill().unwrap();
                let _ = instance.wait(); // necessary on some os

                ip.set_port(ip.port + 1);
            } else {
                break instance;
            }
        };

        log::trace!("Service started on {}", ip.to_string());
        self.services.push(Service {
            instance,
            stdin: write,
            ip: ip,
            pods: Vec::new(),
        });

        Ok(())
    }

    /// Runs a command with the cli and returns it's stdout
    /// Returns (status, stdio, stderr)
    fn cli_command<I, S>(args: I) -> io::Result<(ExitStatus, String, String)>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let output = std::process::Command::new(CLI_BIN).args(args).output()?;
        Ok((
            output.status,
            std::str::from_utf8(&output.stdout).unwrap().to_string(),
            std::str::from_utf8(&output.stderr).unwrap().to_string(),
        ))
    }

    /// Cli commands to create a pod
    fn cli_pod_creation_command(
        network_name: String,
        service_ip: &IpP,
        dir_path: &Path,
        ip: &IpP,
        connect_to: Option<&IpP>,
        pipe_output: bool,
    ) -> Result<std::process::ExitStatus, Box<dyn std::error::Error>> {
        let mut command = std::process::Command::new(CLI_BIN);
        log::trace!("Cli template command.");
        command
            .args(&[
                "template".to_string(),
                "-C".to_string(),
                dir_path.to_string_lossy().to_string(),
            ])
            .stdout(Self::generate_pipe(pipe_output))
            .stderr(Self::generate_pipe(pipe_output))
            .spawn()?
            .wait()?;

        let mut command = std::process::Command::new(CLI_BIN);
        log::trace!("Cli new pod command.");
        Ok(command
            .args({
                let mut args = vec![
                    service_ip.to_string(), // service ip
                    "new".to_string(),
                    network_name, // network name
                    "-C".to_string(),
                    dir_path.to_string_lossy().to_string(),
                    "-i".to_string(),
                    ip.to_string(),
                ];

                if let Some(peer) = connect_to {
                    args.push("-u".to_string());
                    args.push(peer.to_string());
                }
                args
            })
            .stdout(Self::generate_pipe(pipe_output))
            .stderr(Self::generate_pipe(pipe_output))
            .spawn()?
            .wait()?)
    }

    /// Create pod connected to a network for each service running
    /// except if the service already has a pod on that network
    pub fn create_network(
        &mut self,
        network_name: String,
        pipe_output: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let last_pod_ip = self
            .services
            .iter()
            .map(|service| &service.pods)
            .flatten()
            .max_by(|(_, ip, _), (_, ip2, _)| ip.get_ip_last().cmp(&ip2.get_ip_last()))
            .map(|(_, ip, _)| ip.clone());

        self.services
            .iter_mut()
            .fold(last_pod_ip, |conn_to, service| {
                if let Some((_, ip, _)) = service.pods.iter().find(|(nw, _, _)| *nw == network_name)
                {
                    Some(ip.clone())
                } else {
                    let temp_dir = assert_fs::TempDir::new().expect("can't create temp dir");
                    let mut pod_ip = conn_to
                        .clone()
                        .unwrap_or(IpP::try_from(&"127.0.0.1:8080".to_string()).unwrap());
                    pod_ip.set_ip_last(pod_ip.get_ip_last() + 1);

                    let exit_status = Self::cli_pod_creation_command(
                        network_name.clone(),
                        &service.ip,
                        temp_dir.path(),
                        &pod_ip,
                        conn_to.as_ref(),
                        pipe_output,
                    );

                    match exit_status {
                        Ok(status) if status.success() => {
                            service
                                .pods
                                .push((network_name.clone(), pod_ip.clone(), temp_dir));
                            log::trace!("pod created successfully");
                            Some(pod_ip)
                        }
                        Ok(status) => panic!("Error code from the cli: {status}"),
                        Err(e) => {
                            panic!("Cli command to create pod failed: {e}");
                        }
                    }
                }
            });
        Ok(())
    }
}
