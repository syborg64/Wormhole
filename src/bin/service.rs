// In rust we code
// In code we trust
// AgarthaSoftware - 2024

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
use wormhole::config::types::{
    GeneralGlobalConfig, GeneralLocalConfig, GlobalConfig, LocalConfig, RedundancyConfig,
};
use wormhole::pods::whpath::{JoinPath, WhPath};
use wormhole::{
    commands::{self, cli_commands::Cli},
    config,
};
use wormhole::{network::server::Server, pods::declarations::Pod};

enum PodCommand {
    AddPod(Pod),
}

async fn handle_cli_command(
    tx: mpsc::UnboundedSender<PodCommand>,
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) -> Result<(), String> {
    let (mut write, mut read) = ws_stream.split();

    // Attendre un message de la CLI
    if let Some(Ok(message)) = read.next().await {
        let bytes = message.into_data();

        // Désérialisation de la commande CLI
        let cli: Cli = bincode::deserialize(&bytes)
            .map_err(|e| format!("Erreur de désérialisation : {}", e))?;

        // Traitement de la commande et préparation de la réponse
        let response = match cli {
            Cli::Init(pod_args) => {
                match commands::service::init(pod_args).await {
                    Ok((global_config, local_config, server, mount_point)) => {
                        let new_pod = Pod::new(
                            mount_point,
                            1,
                            global_config.general.peers,
                            server.clone(),
                            local_config.general.address,
                        )
                        .await
                        .map_err(|e| format!("Erreur lors de la création du pod : {}", e))?;

                        // Envoi de la commande pour ajouter le pod
                        tx.send(PodCommand::AddPod(new_pod))
                            .map_err(|e| format!("Erreur lors de l'envoi de PodCommand : {}", e))?;

                        "Pod ajouté".to_string()
                    }
                    Err(e) => {
                        let error_msg = format!("Erreur d'initialisation : {}", e);
                        error!("{}", error_msg); // Log l'erreur dans le terminal
                        error_msg // Renvoyer l'erreur à la CLI
                    }
                }
            }
            _ => "Commande non reconnue".to_string(),
        };

        // Envoi de la réponse à la CLI
        write
            .send(Message::Text(response))
            .await
            .map_err(|e| format!("Erreur lors de l'envoi de la réponse : {}", e))?;
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
    let mut pods = Vec::new();

    // Lancer la tâche centrale
    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            match command {
                PodCommand::AddPod(pod) => {
                    pods.push(pod);
                    println!("Pods created: {:?}", pods);
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

//----------------------------------------------------------//

// async fn main1() {
//     let mut pods: Vec<Pod> = Vec::new();

//     env_logger::init();
//     let mount: PathBuf = env::args()
//         .nth(1)
//         .unwrap_or("./virtual/".to_string())
//         .into();

//     let mut global_config_path = mount.clone();
//     global_config_path.push(".global_config.toml");

//     #[cfg(target_os = "windows")]
//     match winfsp_init() {
//         Ok(_token) => println!("got fsp token!"),
//         Err(err) => {
//             println!("fsp error: {:?}", err);
//             std::process::exit(84)
//         }
//     }

//     let mut local_config_path = mount.clone();
//     local_config_path.push(".local_config.toml");

//     let mut args_other_addresses: Vec<String> = env::args().collect();
//     if args_other_addresses.len() >= 3 {
//         args_other_addresses.remove(0);
//         args_other_addresses.remove(0);
//         args_other_addresses.remove(0);
//     };

//     let mut global_config: GlobalConfig = config::parse_toml_file(global_config_path.as_str())
//         .unwrap_or(GlobalConfig {
//             general: GeneralGlobalConfig {
//                 peers: args_other_addresses,
//                 ignore_paths: vec![],
//             },
//             redundancy: Some(RedundancyConfig { number: 3 }),
//         });

//     global_config
//         .general
//         .ignore_paths
//         .push(".local_config.toml".to_string());

//     let local_config: LocalConfig = match config::parse_toml_file(local_config_path.as_str()) {
//         Err(error) => {
//             log::warn!("Local Config Not Found: {error}");

//             let own_addr = match env::args().nth(2) {
//                 Some(address) => address,
//                 None => {
//                     log::error!("Local config missing and own Address missing from args");
//                     return;
//                 }
//             };

//             LocalConfig {
//                 general: GeneralLocalConfig {
//                     name: own_addr.clone(),
//                     address: own_addr,
//                 },
//             }
//         }
//         Ok(found) => found,
//     };

//     log::info!("WHConfig: {global_config:?} {local_config:?}");

//     let server = Arc::new(Server::setup(&local_config.general.address).await);

//     pods.push(
//         Pod::new(
//             WhPath::from(mount.as_path()),
//             1,
//             global_config.general.peers,
//             server.clone(),
//             local_config.general.address,
//         )
//         .await
//         .expect("failed to create the pod"),
//     );

//     // let local_cli_handle = tokio::spawn(local_cli_watchdog());
//     // log::info!("Started");
//     // local_cli_handle.await.unwrap(); // keeps the main process alive until interruption from this watchdog;
//     // log::info!("Stopping");
// }
