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

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
#[cfg(target_os = "windows")]
use winfsp::winfsp_init;
use wormhole::commands::cli_commands::{Mode, StatusPodArgs};
use wormhole::commands::PodCommand;
use wormhole::commands::{self, cli_commands::Cli};
use wormhole::error::{CliError, CliResult, CliSuccess, WhError, WhResult};
use wormhole::pods::pod::{Pod, PodStopError};

/*
async fn response_to_cli(wrtier: SplitSink<WebSocketStream<TcpStream>, Message>,response_cmd: CliResult) {
    pods = match response_cmd {
        Ok(CliSuccess::PodCreated(pod)) => {
            info!("Pod created or joined a network");
            let name = pod.get_name().to_string();
            pods.insert(name.clone(), pod);
            let _ = writer
                .send(Message::Text(
                    CliSuccess::WithData {
                        message: String::from("Pod created with success: "),
                        data: name,
                    }
                    .to_string(),
                ))
                .await
                .map_err(|e| format!("Response send error: {}", e));
            pods
        }
        Ok(CliSuccess::Message(msg)) => {
            let _ = writer
                .send(Message::Text(CliSuccess::Message(msg).to_string()))
                .await
                .map_err(|e| format!("Response send error: {}", e));
            pods
        }
        Ok(CliSuccess::WithData { message, data }) => {
            let _ = writer
                .send(Message::Text(
                    CliSuccess::WithData { message, data }.to_string(),
                ))
                .await
                .map_err(|e| format!("Response send error: {}", e));
            pods
        }
        Err(err) => {
            let _ = writer
                .send(Message::Text(err.to_string()))
                .await
                .map_err(|e| format!("Response send error: {}", e));
            pods
        }
    };
}
 */
type CliTcpWriter =
    SplitSink<WebSocketStream<tokio::net::TcpStream>, tokio_tungstenite::tungstenite::Message>;

async fn handle_cli_command(
    mut pods: HashMap<String, Pod>,
    command: Cli,
    mut writer: CliTcpWriter,
) -> HashMap<String, Pod> {
    let response_command = match command {
        Cli::New(pod_args) => match commands::service::new(pod_args).await {
            Ok(pod) => {
                let name = pod.get_name().to_string();
                pods.insert(name.clone(), pod);
                Ok(CliSuccess::WithData {
                    message: String::from("Pod created with success"),
                    data: name,
                })
            }
            Err(e) => Err(e),
        },
        Cli::Start(pod_args) => commands::service::start(pod_args).await,
        Cli::Stop(pod_args) => {
            if let Some(pod) = pod_args.name.and_then(|name| pods.remove(&name)) {
                commands::service::stop(pod).await
            } else {
                log::warn!("(TODO) Stopping a pod by path is not yet implemented");
                Err(CliError::InvalidCommand)
            }
        }
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
                Err(CliError::PodRemovalFailed {
                    reason: String::from("Pod not found"),
                })
            }
        }
        Cli::Restore(resotre_args) => commands::service::restore(resotre_args).await,
        _ => Err(CliError::InvalidCommand),
    };
    let string_output = response_command.map_or_else(|e| e.to_string(), |a| a.to_string());
    match writer.send(Message::Text(string_output)).await {
        Ok(()) => log::debug!("Sent answer to cli"),
        Err(err) => log::error!("Message can't send to cli: {}", err),
    }
    pods
}

async fn get_cli_command(stream: tokio::net::TcpStream) -> WhResult<(Cli, CliTcpWriter)> {
    // Accept the TCP stream as a WebSocket stream
    let ws_stream = match accept_async(stream).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("get_cli_command: can't accept tcp stream: {}", e);
            return Err(WhError::NetworkDied {
                called_from: "get_cli_command::accept_tcp_stream".to_owned(),
            });
        }
    };
    let (writer, mut reader) = ws_stream.split();

    // Read the first message from the stream
    let message_data = match reader.next().await {
        Some(Ok(msg)) => msg.into_data(),
        Some(Err(e)) => {
            log::error!("get_cli_command: invalid message: {}", e);
            return Err(WhError::NetworkDied {
                called_from: "get_cli_command".to_owned(),
            });
        }
        None => {
            log::error!("get_cli_command: can't get message from tcp stream");
            return Err(WhError::NetworkDied {
                called_from: "get_cli_command".to_owned(),
            });
        }
    };

    // Deserialize the message data into a Cli object
    let cmd = match bincode::deserialize(&message_data) {
        Ok(c) => c,
        Err(e) => {
            log::error!("get_cli_command: failed to deserialize message: {}", e);
            return Err(WhError::NetworkDied {
                called_from: "get_cli_command::deserialize_message".to_owned(),
            });
        }
    };

    Ok((cmd, writer))
}

/// Listens for CLI calls and launch one tcp instance per cli command
async fn start_cli_listener(
    mut pods: HashMap<String, Pod>,
    ip: String,
    mut interrupt_rx: UnboundedReceiver<()>,
) -> HashMap<String, Pod> {
    println!("Écoute des commandes CLI sur {}", ip);
    let listener = TcpListener::bind(&ip)
        .await
        .expect(format!("Échec de la liaison au port {}", &ip).as_str());
    info!("Écoute des commandes CLI sur {}", ip);

    while let Some(Ok((stream, _))) = tokio::select! {
        v = listener.accept() => Some(v),
        _ = interrupt_rx.recv() => None,
    } {
        let (command, writer) = match get_cli_command(stream).await {
            Ok(cmd) => cmd,
            Err(e) => {
                log::error!("cli_listener: error on getting command: {e}");
                continue;
            }
        };
        pods = handle_cli_command(pods, command, writer).await;
    }
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
    let (interrupt_tx, interrupt_rx) = mpsc::unbounded_channel::<()>();
    let mut pods: HashMap<String, Pod> = HashMap::new();

    // Lancer la tâche centrale

    env_logger::init();
    let ip: String = env::args()
        .nth(1)
        .unwrap_or("127.0.0.1:8081".to_string())
        .into();
    println!("Starting service on {}", ip);

    let terminal_handle = tokio::spawn(terminal_watchdog(interrupt_tx));
    let cli_airport = tokio::spawn(start_cli_listener(pods, ip, interrupt_rx));
    log::info!("Started");

    pods = cli_airport
        .await
        .expect("main: cli_airport didn't join properly");
    terminal_handle
        .abort();

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
pub async fn terminal_watchdog(tx: UnboundedSender<()>) {
    let mut stdin = tokio::io::stdin();
    let mut buf = vec![0; 1024];

    while let Ok(read) = tokio::io::AsyncReadExt::read(&mut stdin, &mut buf).await {
        // NOTE -  on ctrl-D -> quit
        match read {
            0 => {
                println!("Quiting!");
                let _ = tx.send(());
                return;
            }
            _ => (),
        };
    }
}
