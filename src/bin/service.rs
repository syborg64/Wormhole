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
use tokio::sync::oneshot::Sender;
use tokio::task::JoinError;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
#[cfg(target_os = "windows")]
use winfsp::winfsp_init;
use wormhole::commands::cli_commands::StatusPodArgs;
use wormhole::commands::PodCommand;
use wormhole::commands::{self, cli_commands::Cli};
use wormhole::error::CliError;
use wormhole::pods::pod::Pod;
use wormhole::pods::pod::PodStopError;

async fn handle_cli_command(
    tx: mpsc::UnboundedSender<PodCommand>,
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) -> Result<(), String> {
    let (mut writer, mut reader) = ws_stream.split();

    if let Some(Ok(message)) = reader.next().await {
        let message_bytes = message.into_data();

        let cli_command: Cli = bincode::deserialize(&message_bytes)
            .map_err(|e| format!("Deserialization error: {}", e))?;

        let response_text = match cli_command {
            Cli::New(pod_args) => commands::service::new(tx.clone(), pod_args).await,
            Cli::Start(pod_args) => commands::service::start(tx.clone(), pod_args).await,
            Cli::Stop(pod_args) => commands::service::stop(tx.clone(), pod_args).await,
            Cli::Remove(remove_arg) => commands::service::remove(tx, remove_arg).await,
            _ => Err(CliError::InvalidCommand),
        };

        let message = match response_text {
            Ok(success) => success.to_string(),
            Err(error) => error.to_string(),
        };

        writer
            .send(Message::Text(message))
            .await
            .map_err(|e| format!("Response send error: {}", e))?;
    }

    Ok(())
}

async fn handle_cli(stream: tokio::net::TcpStream, tx: mpsc::UnboundedSender<PodCommand>) {
    match accept_async(stream).await {
        Ok(ws_stream) => {
            // Gérer la commande et logger les erreurs si elles surviennent
            if let Err(e) = handle_cli_command(tx, ws_stream).await {
                error!("Erreur dans handle_cli_command : {}", e);
            }
        }
        Err(e) => error!("Erreur lors de la négociation WebSocket : {}", e),
    }
}

// Gestion de l'écoute des connexions CLI
async fn start_cli_listener(tx: mpsc::UnboundedSender<PodCommand>, ip: String) {
    println!("Écoute des commandes CLI sur {}", ip);
    let listener = TcpListener::bind(&ip)
        .await
        .expect(format!("Échec de la liaison au port {}", &ip).as_str());
    info!("Écoute des commandes CLI sur {}", ip);

    while let Ok((stream, _)) = listener.accept().await {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            handle_cli(stream, tx_clone).await;
        });
    }
}

async fn main_cli_airport(
    mut rx: UnboundedReceiver<PodCommand>,
    mut pods: HashMap<String, Pod>,
) -> HashMap<String, Pod> {
    while let Some(command) = rx.recv().await {
        match command {
            PodCommand::NewPod(pod) => {
                info!("Pod created or joined a network");
                pods.push(pod);
            }
            PodCommand::StartPod(start_args) => {
                info!("Starting pod: {:?}", start_args);
                todo!("Check if pod existe and start it based one his name or path")
            }
            PodCommand::StopPod(StatusPodArgs { name, path: _ }, responder) => {
                let _ = responder.send(
                    name.and_then(|name| pods.get(&name))
                        .ok_or(PodStopError::PodNotRunning)
                        .and_then(|pod| pod.stop())
                        .map(|()| "Pod was stopped.".to_string()),
                );
            }
            PodCommand::RemovePod(args) => {
                info!("Pod removed: {:?}", args);

                let pod = pods.iter().find(|pod| {
                    // Vérifie si le nom correspond (si présent dans args)
                    let name_matches = args
                        .name
                        .as_ref()
                        .map_or(false, |arg_name| arg_name == pod.get_name());
                    // Vérifie si le chemin correspond (si présent dans args)
                    let path_matches = args
                        .path
                        .as_ref()
                        .map_or(false, |arg_path| arg_path == pod.get_mount_point());
                    // Retourne vrai si au moins une condition est remplie
                    name_matches || path_matches
                });
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
    let cli_listener = tokio::spawn(start_cli_listener(tx, ip));
    let cli_airport = tokio::spawn(main_cli_airport(rx, pods));
    log::info!("Started");

    let pods = tokio::select! {
        pods = cli_airport => Some(pods.expect("main: cli_airport didn't join properly")),
        _ = terminal_handle => None,
        _ = cli_listener => None,
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
