use crate::pods::{pod::Pod, whpath::WhPath};
use clap::{Args, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
pub enum PodCommand {
    NewPod(String, Pod),
    StartPod(StatusPodArgs),
    StopPod(
        StatusPodArgs,
        tokio::sync::oneshot::Sender<Result<String, crate::pods::pod::PodStopError>>,
    ),
    RemovePod(RemoveArgs),
    Interrupt,
}

#[derive(Debug, Parser, Serialize, Deserialize)] // requires `derive` feature
#[command(name = "wormhole")]
#[command(bin_name = "wormhole")]
pub enum Cli {
    /// start the service
    Start(StatusPodArgs),
    /// stop the service
    Stop(StatusPodArgs),
    /// create a new network (template)
    Template(TemplateArg),
    /// create a new pod and join a network if he have peers in arguments or create a new network
    New(PodArgs),
    /// inspect a pod with its configuration, connections, etc
    Inspect,
    /// remove a pod from its network
    Remove(RemoveArgs),
    /// reload a pod
    Reload(PodArgs),
}

#[derive(Debug, clap::Args, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct PodArgs {
    /// Name of the pod
    pub name: String,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    pub path: Option<WhPath>,
    /// network url as <address of node to join from> + ':' + <network name>'
    #[arg()]
    pub url: Option<String>,
    /// additional hosts to try to join from as a backup
    #[arg()]
    pub additional_hosts: Option<Vec<String>>,
}

#[derive(Debug, clap::Args, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct StatusPodArgs {
    /// Name of the pod for updating status pod. If the name equal 'None' the name will be read from the current directory
    pub name: Option<String>,
    /// Path is used uniquely if the pod name is 'None'
    pub path: Option<WhPath>,
}

#[derive(Debug, clap::Args, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct TemplateArg {
    /// name of the network to create
    #[arg()]
    pub name: Option<String>,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    pub path: Option<WhPath>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Mode {
    /// Simply remove the pod from the network without losing any data from the network
    /// and leaving behind any data that was stored on the pod
    Simple,
    /// Remove the pod from the network without losing any data on the network,
    /// and clone all data from the network into the folder where the pod was
    /// making this folder into a real folder
    Clone,
    /// Remove the pod from the network and delete any data that was stored in the pod
    Clean,
    /// Remove this pod from the network without distributing its data to other nodes
    Take,
}

// Structure RemoveArgs modifiée
#[derive(Debug, Args, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct RemoveArgs {
    /// Name of the deleted pod
    pub name: Option<String>,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    pub path: Option<WhPath>,
    /// Mode for pod removal
    #[arg(long, default_value = "simple")]
    pub mode: Mode,
}
