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
use std::env;

use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use log::{error, info};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
#[cfg(target_os = "windows")]
use winfsp::winfsp_init;
use wormhole::commands::cli_commands::Mode;
use wormhole::commands::PodCommand;
use wormhole::commands::{self, cli_commands::Cli};
use wormhole::error::CliError;

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

#[tokio::main]
async fn main() {
    // Créer un canal pour envoyer des commandes
    let (tx, mut rx) = mpsc::unbounded_channel::<PodCommand>();
    let mut pods = Vec::new();

    // Lancer la tâche centrale
    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            match command {
                PodCommand::NewPod(pod) => {
                    info!("Pod created or joined a network");
                    pods.push(pod);
                }
                PodCommand::StartPod(start_args) => {
                    info!("Pod started: {:?}", start_args);
                    todo!("Check if pod existe and start it based one his name or path")
                }
                PodCommand::StopPod(stop_args) => {
                    info!("Pod stopped: {:?}", stop_args);
                    todo!("Check if pod existe and stop it based one his name or path")
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
    
    // env_logger::init();
    // let mount: PathBuf = env::args()
    //     .nth(1)
    //     .unwrap_or("./virtual/".to_string())
    //     .into();

    // let mut global_config_path = mount.clone();
    // global_config_path.push(".global_config.toml");

    // #[cfg(target_os = "windows")]
    // match winfsp_init() {
    //     Ok(_token) => println!("got fsp token!"),
    //     Err(err) => {
    //         println!("fsp error: {:?}", err);
    //         std::process::exit(84)
    //     }
    // }

    // let mut local_config_path = mount.clone();
    // local_config_path.push(".local_config.toml");

    // let mut args_other_addresses: Vec<String> = env::args().collect();
    // if args_other_addresses.len() >= 3 {
    //     args_other_addresses.remove(0);
    //     args_other_addresses.remove(0);
    //     args_other_addresses.remove(0);
    // };

    // let mut global_config: GlobalConfig = config::parse_toml_file(global_config_path.as_str())
    //     .unwrap_or(GlobalConfig {
    //         general: GeneralGlobalConfig {
    //             peers: vec![],
    //             ignore_paths: vec![],
    //         },
    //         redundancy: RedundancyConfig { number: 2 },
    //     });

    // for address in args_other_addresses {
    //     global_config.general.peers.push(address);
    // }
    // global_config.general.peers.sort();
    // global_config.general.peers.dedup();

    // global_config
    //     .general
    //     .ignore_paths
    //     .push(".local_config.toml".to_string());

    // let local_config: LocalConfig = match config::parse_toml_file(local_config_path.as_str()) {
    //     Err(error) => {
    //         log::warn!("Local Config Not Found: {error}");

    //         let own_addr = match env::args().nth(2) {
    //             Some(address) => address,
    //             None => {
    //                 log::error!("Local config missing and own Address missing from args");
    //                 return;
    
    // let server = Arc::new(Server::setup(&local_config.general.address).await);

    // pods.push(
    //     Pod::new(
    //         global_config,
    //         WhPath::from(mount.as_path()),
    //         1,
    //         server.clone(),
    //         local_config.general.address,
    //     )
    //     .await
    //     .expect("failed to create the pod"),
    // );

    // let local_cli_handle = tokio::spawn(local_cli_watchdog());
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
