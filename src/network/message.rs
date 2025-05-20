use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    error::WhResult,
    pods::arbo::{ArboIndex, Inode, InodeId, Metadata},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServiceMessage {
    Register(Address),
    ToPod(String, MessageContent)
}

/// Message Content
/// Represent the content of the intern message but is also the struct sent
/// through the network
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageContent {
    Register(Address),
    Remove(InodeId),
    Inode(Inode),
    RequestFile(InodeId, Address),
    PullAnswer(InodeId, Vec<u8>),
    Rename(InodeId, InodeId, String, String), //Parent, New Parent, Name, New Name
    EditHosts(InodeId, Vec<Address>),
    AddHosts(InodeId, Vec<Address>),
    RemoveHosts(InodeId, Vec<Address>),
    EditMetadata(InodeId, Metadata, Address),
    SetXAttr(InodeId, String, Vec<u8>),
    RemoveXAttr(InodeId, String),
    RequestFs,
    RequestPull(InodeId),
    Disconnect(Address),
    // Arbo, peers, .global_config
    FsAnswer(FileSystemSerialized, Vec<Address>, Vec<u8>),
}

pub type MessageAndStatus = (MessageContent, Option<UnboundedSender<WhResult<()>>>);

pub type Address = String;

/// Message Coming from Network
/// Messages recived by peers, forwared to [crate::network::watchdogs::network_file_actions]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FromNetworkMessage {
    pub origin: Address,
    pub content: MessageContent,
}

/// Message Going To Network
/// Messages sent from fuser to process communicating to the peers
#[derive(Debug)]
pub enum ToNetworkMessage {
    BroadcastMessage(MessageContent),
    SpecificMessage(MessageAndStatus, Vec<Address>),
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSystemSerialized {
    pub fs_index: ArboIndex,
    pub next_inode: InodeId,
}
