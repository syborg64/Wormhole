use fuser::{FileAttr, FileType};
use openat::Dir;
use std::{collections::HashMap, path::PathBuf, time::UNIX_EPOCH};
use tokio::sync::mpsc::UnboundedSender;

use crate::network::message::NetworkMessage;

mod helpers;
pub mod readers;
pub mod whpath;
pub mod writers;

// (inode_number, (Type, Original path))
pub type FsIndex = HashMap<u64, (fuser::FileType, PathBuf)>;

// will keep all the necessary info to provide real
// data to the fuse lib
// For now this is given to the fuse controler on creation and we do NOT have
// ownership during the runtime.
pub struct Provider {
    pub next_inode: u64,
    pub index: FsIndex,
    pub local_source: PathBuf,
    pub metal_handle: Dir,
    pub tx: UnboundedSender<NetworkMessage>,
}

// will soon be replaced once the dev continues
const TEMPLATE_FILE_ATTR: FileAttr = FileAttr {
    ino: 2,   // required to be correct
    size: 13, // required to be correct
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile, // required to be correct
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

impl Provider {}
