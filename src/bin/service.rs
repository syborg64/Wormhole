// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::collections::HashMap;
/**DOC
 * Important variables to know :
 * nfa_rx - nfa_tx
 *  Use nfa_tx to send a file related message to the newtork_file_actions function
 *
 * Important functions to know :
 *
 * local_cli_watchdog
 *  this is the handle linked to the terminal, that will terminate the
 *  program if CTRL-D
 *
 * newtork_file_actions
 *  reads a message (supposely emitted by a peer) related to files actions
 *  and execute instructions on the disk
 */
use std::env;

use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use log::{error, info};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
#[cfg(target_os = "windows")]
use winfsp::winfsp_init;
use wormhole::commands::cli_commands::{Mode, StatusPodArgs};
use wormhole::commands::PodCommand;
use wormhole::commands::{self, cli_commands::Cli};
use wormhole::error::{CliError, CliSuccess};
use wormhole::pods::pod::{Pod, PodStopError};

async fn handle_cli_command(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    mut pods: HashMap<String, Pod>,
) -> HashMap<String, Pod> {
    let (mut writer, mut reader) = ws_stream.split();

    if let Some(Ok(message)) = reader.next().await {
        let message_bytes = message.into_data();

        
        let cli_command: Cli = match bincode::deserialize(&message_bytes) {
            Ok(cli) => cli,
            Err(e) => {
                error!("Deserialization error: {}", e);
                return pods;
            },   
        };

        let response_command = match cli_command {
            Cli::New(pod_args) => commands::service::new(pod_args).await,
            Cli::Start(pod_args) => commands::service::start(pod_args).await,
            Cli::Stop(pod_args) => {
                if let Some(pod) = pod_args.name.and_then(|name| pods.remove(&name)) {
                    commands::service::stop(pod).await
                } else {
                    log::warn!("(TODO) Stopping a pod by path is not yet implemented");
                    Err(CliError::InvalidCommand)
                }
            },
            Cli::Remove(remove_arg) => {
                let opt = if let Some(name) = remove_arg.clone().name {
                    // pods.get(&name)
                    pods.remove(&name)
                } else if let Some(path) = remove_arg.clone().path {
                    let key_to_remove = pods
                        .iter()
                        .find(|(_, pod)| pod.get_mount_point() == &path)
                        .map(|(key, _)| key.clone());

                    key_to_remove.and_then(|key| pods.remove(&key))
                } else {
                    log::error!("No pod name nor path were provided by RemovePod command");
                    None
                };
                if let Some(pod) = opt {
                    commands::service::remove(remove_arg, pod).await
                } else {
                    log::info!("Pod not found");
                    Err(CliError::PodRemovalFailed { reason: String::from("Pod not found") })
                }
            },
            Cli::Restore(resotre_args) => commands::service::restore(resotre_args).await,
            _ => Err(CliError::InvalidCommand),
        };

        pods = match response_command {
            Ok(CliSuccess::PodCreated(pod)) => {
                info!("Pod created or joined a network");
                let name = pod.get_name().to_string();
                pods.insert(name.clone(), pod);
                let _ = writer
                            .send(Message::Text(CliSuccess::WithData { message: String::from("Pod created with success: "), data: name }.to_string()))
                            .await
                            .map_err(|e| format!("Response send error: {}", e));
                pods
            },
            Ok(CliSuccess::Message(msg)) => {
                let _ = writer
                            .send(Message::Text(CliSuccess::Message(msg).to_string()))
                            .await
                            .map_err(|e| format!("Response send error: {}", e));
                pods
            },
            Ok(CliSuccess::WithData { message, data }) => {
                let _ = writer
                            .send(Message::Text(CliSuccess::WithData { message, data }.to_string()))
                            .await
                            .map_err(|e| format!("Response send error: {}", e));
                pods
            },
            Err(err) => {
                let _ = writer
                            .send(Message::Text(err.to_string()))
                            .await
                            .map_err(|e| format!("Response send error: {}", e));
            pods
        },
        };
    }
    pods
}

async fn handle_cli(stream: tokio::net::TcpStream, mut pods: HashMap<String, Pod>) -> HashMap<String, Pod> {
    match accept_async(stream).await {
        Ok(ws_stream) => pods = handle_cli_command(ws_stream, pods).await,
        Err(e) => error!("Erreur lors de la négociation WebSocket : {}", e),
    }
    pods
}

// Gestion de l'écoute des connexions CLI
async fn start_cli_listener(
    pods: HashMap<String, Pod>,
    ip: String) -> HashMap<String, Pod> {
    println!("Écoute des commandes CLI sur {}", ip);
    let listener = TcpListener::bind(&ip)
        .await
        .expect(format!("Échec de la liaison au port {}", &ip).as_str());
    info!("Écoute des commandes CLI sur {}", ip);

    while let Ok((stream, _)) = listener.accept().await {
        let cli_airport = tokio::spawn(handle_cli(stream, pods));
        let pods = tokio::select! {
            p = cli_airport => p.expect("start_cli_listener: handle_cli didn't join properly"), 
            
        };
        return pods;
    };
    pods
}

async fn main_cli_airport(
    mut rx: UnboundedReceiver<PodCommand>,
    mut pods: HashMap<String, Pod>,
) -> HashMap<String, Pod> {
    while let Some(command) = rx.recv().await {
        match command {
            PodCommand::NewPod(name, pod) => {
                info!("Pod created or joined a network");
                pods.insert(name, pod);
            }
            PodCommand::StartPod(start_args) => {
                info!("Starting pod: {:?}", start_args);
                todo!("Check if pod existe and start it based one his name or path")
            }
            PodCommand::StopPod(StatusPodArgs { name, path: _ }, responder) => {
                let status = if let Some(pod) = name.and_then(|name| pods.remove(&name)) {
                    tokio::task::spawn_blocking(move || pod.stop())
                        .await
                        .expect("pod stop: can't spawn blocking task")
                        .map(|()| "Pod was stopped.".to_string())
                } else {
                    // TODO - allow deletion by path
                    log::warn!("(TODO) Stopping a pod by path is not yet implemented");
                    Err(PodStopError::PodNotRunning)
                };
                let _ = responder.send(status);
            }
            PodCommand::RemovePod(args) => {
                info!("Pod removed: {:?}", args);

                let pod = if let Some(name) = args.name {
                    pods.get(&name)
                } else if let Some(path) = args.path {
                    pods.values().find(|pod| *pod.get_mount_point() == path)
                } else {
                    log::error!("No pod name nor path were provided by RemovePod command");
                    None
                };

                if pod.is_none() {
                    info!("Pod not found");
                } else {
                    match args.mode {
                        Mode::Simple => {
                            //TODO - stop the pod
                            //TODO - delete the pod
                            todo!()
                        }
                        Mode::Clone => {
                            //TODO - clone all data into a folder
                            //TODO - stop the pod
                            //TODO - delete the pod
                            todo!()
                        }
                        Mode::Clean => {
                            //TODO - stop the pod
                            //TODO - delete all data
                            //TODO - delete the pod
                            todo!()
                        }
                        Mode::Take => {
                            //TODO - stop the pod without distributing its data
                            //TODO - delete the pod
                            todo!()
                        }
                    }
                }
                todo!("Check if pod existe and remove it based one his name or path")
            }
            PodCommand::Interrupt => break,
        }
    }
    pods
}

#[tokio::main]
async fn main() {
    // Créer un canal pour envoyer des commandes
    let (tx, rx) = mpsc::unbounded_channel::<PodCommand>();
    let pods: HashMap<String, Pod> = HashMap::new();

    // Lancer la tâche centrale

    env_logger::init();
    let ip: String = env::args()
        .nth(1)
        .unwrap_or("127.0.0.1:8081".to_string())
        .into();
    println!("Starting service on {}", ip);

    let terminal_handle = tokio::spawn(terminal_watchdog(tx.clone()));
    let cli_airport = tokio::spawn(start_cli_listener(pods, ip));
    // let cli_airport = tokio::spawn(main_cli_airport(rx, pods));
    log::info!("Started");

    let pods = tokio::select! {
        // pods = cli_airport => Some(pods.expect("main: cli_airport didn't join properly")),
        pods = cli_airport => Some(pods.expect("main: cli_airport didn't join properly")),
        _ = terminal_handle => None,
    }
    .expect("runtime returned unexpectedly");

    log::info!("Stopping");
    for (name, pod) in pods.iter() {
        match pod.stop() {
            Ok(()) => log::info!("Stopped pod {name}"),
            Err(e) => log::error!("Pod {name} can't be stopped: {e}"),
        }
    }
    log::info!("Stopped");
}

// NOTE - old watchdog brought here for debug purposes
pub async fn terminal_watchdog(tx: UnboundedSender<PodCommand>) {
    let mut stdin = tokio::io::stdin();
    let mut buf = vec![0; 1024];

    while let Ok(read) = tokio::io::AsyncReadExt::read(&mut stdin, &mut buf).await {
        // NOTE -  on ctrl-D -> quit
        match read {
            0 => {
                println!("Quiting!");
                let _ = tx.send(PodCommand::Interrupt);
            }
            _ => (),
        };
    }
}
