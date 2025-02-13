use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::pods::arbo::{ArboIndex, Inode, InodeId, Metadata};

/// Message Content
/// Represent the content of the intern message but is also the struct sent
/// through the network
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageContent {
    Remove(InodeId),
    Inode(Inode, InodeId),
    RequestFile(InodeId, Address),
    PullAnswer(InodeId, Vec<u8>),
    RequestFs(Address),
    EditHosts(InodeId, Vec<Address>),
    EditMetadata(InodeId, Metadata, Address),
    FsAnswer(FileSystemSerialized),
}

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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToNetworkMessage {
    BroadcastMessage(MessageContent),
    SpecificMessage(MessageContent, Vec<Address>),
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSystemSerialized {
    pub fs_index: ArboIndex,
    pub next_inode: InodeId,
}
