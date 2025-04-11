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
use std::{env, path::PathBuf, sync::Arc};

use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use log::{error, info};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
#[cfg(target_os = "windows")]
use winfsp::winfsp_init;
use wormhole::commands::PodCommand;
use wormhole::config::types::{
    GeneralGlobalConfig, GeneralLocalConfig, GlobalConfig, LocalConfig, RedundancyConfig,
};
use wormhole::pods::whpath::{JoinPath, WhPath};
use wormhole::{
    commands::{self, cli_commands::Cli},
    config,
};
use wormhole::{network::server::Server, pods::pod::Pod};

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
            Cli::Init(pod_args) => commands::service::init(tx.clone(), pod_args).await,
            Cli::Join(join_args) => commands::service::join(tx.clone(), join_args).await,
            Cli::Start(pod_args) => commands::service::start(tx.clone(), pod_args).await,
            Cli::Stop(pod_args) => commands::service::stop(tx.clone(), pod_args).await,
            _ => Err("Unrecognized command".to_string()),
        };

        writer
            .send(Message::Text(response_text.unwrap_or_else(|e| e)))
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

#[tokio::main]
async fn main() {
    // Créer un canal pour envoyer des commandes
    let (tx, mut rx) = mpsc::unbounded_channel::<PodCommand>();
    let mut pods: HashMap<String, Pod> = HashMap::new();

    // Lancer la tâche centrale
    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            match command {
                PodCommand::AddPod(name, pod) => {
                    info!("Pod created: {:?}", pod);
                    pods.insert(name, pod);
                }
                PodCommand::JoinPod(name, pod) => {
                    info!("Pod created and joined a network: {:?}", pod);
                    pods.insert(name, pod);
                }
                PodCommand::StartPod(start_args) => {
                    info!("Starting pod: {:?}", start_args);
                    todo!("Check if pod existe and start it based one his name or path")
                }
                PodCommand::StopPod(stop_args) => {
                    if let Some(name) = stop_args.name {
                        info!("Stopping pod: {}", name);
                        if let Some(pod) = pods.get(&name) {
                            pod.stop();
                        } else {
                            log::error!("Pod {name} not found. Can't be stopped.")
                        }
                    } else {
                        log::error!("stopping pod without name is not supported.");
                    }
                }
            }
        }
    });

    env_logger::init();
    let ip: String = env::args()
        .nth(1)
        .unwrap_or("127.0.0.1:8081".to_string())
        .into();
    println!("Starting service on {}", ip);
    let tx_clone = tx.clone();
    tokio::spawn(start_cli_listener(tx_clone, ip));

    let terminal_handle = tokio::spawn(terminal_watchdog());
    log::info!("Started");
    terminal_handle.await.unwrap(); // keeps the main process alive until interruption from this watchdog;
    log::info!("Stopping");
}

// NOTE - old watchdog brought here for debug purposes
pub async fn terminal_watchdog() {
    let mut stdin = tokio::io::stdin();
    let mut buf = vec![0; 1024];

    loop {
        let read = tokio::io::AsyncReadExt::read(&mut stdin, &mut buf).await;

        // NOTE -  on ctrl-D -> quit
        match read {
            Err(_) | Ok(0) => {
                println!("Quiting!");
                break;
            }
            _ => (),
        };
    }
}
