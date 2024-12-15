use clap::builder::TypedValueParser;
use fs_attr::FsAttr;
use fuser::{FileAttr, FileType, TimeOrNow};
use futures_util::TryFutureExt;
use openat::Dir;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::network::message::{Address, File, ToNetworkMessage};

pub mod fs_attr;
mod helpers;
pub mod readers;
pub mod whpath;
pub mod writers;

/// Ino is represented by an u64
pub type Ino = u64;

pub type Hosts = Vec<Address>;

/// Hashmap containing file system data
/// (inode_number, (Type, Original path, Hosts))
pub type FsIndex = HashMap<Ino, Fs>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// Should be extended until meeting [fuser::FileType]

pub struct Fs {
    pub entry: FsEntry,
    attr: FsAttr,
}

impl Fs {
    pub fn new(entry: FsEntry) -> Fs {
        let file_type = match entry {
            FsEntry::File(_, _) => FileType::RegularFile,
            FsEntry::Directory(_) => FileType::Directory,
        };

        let attr = FsAttr::new(file_type);
        Fs { entry, attr }
    }

    pub fn set_fs_attr(
        &mut self,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        _crtime: Option<SystemTime>,
        flags: Option<u32>,
    ) {
        let kind = mode.unwrap_or(self.attr.get_allpermission());
        let uid = uid.unwrap_or(self.attr.get_uid());
        let gid = gid.unwrap_or(self.attr.get_gid());
        let size = size.unwrap_or(self.attr.get_size_in_bytes());
        let ctime = _ctime.unwrap_or(self.attr.get_last_change());
        let crtime = _crtime.unwrap_or(self.attr.get_creation_time());
        let flags = flags.unwrap_or(self.attr.get_flags());
        let atime = match _atime {
            Some(TimeOrNow::Now) => SystemTime::now(),
            Some(TimeOrNow::SpecificTime(time)) => time,
            None => self.attr.get_last_access(),
        };
        let mtime = match _mtime {
            Some(TimeOrNow::Now) => SystemTime::now(),
            Some(TimeOrNow::SpecificTime(time)) => time,
            None => self.attr.get_last_modif(),
        };
        self.attr
            .set_file_attr(kind, uid, gid, size, atime, mtime, ctime, crtime, flags);
    }

    pub fn get_fs_attr(&self) -> FileAttr {
        self.attr.get_file_attr()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FsEntry {
    File(PathBuf, Hosts),
    Directory(PathBuf),
}

impl FsEntry {
    pub fn get_path(&self) -> &PathBuf {
        match self {
            FsEntry::File(path, _) => path,
            FsEntry::Directory(path) => path,
        }
    }

    pub fn get_name(&self) -> io::Result<&OsStr> {
        match Path::new(self.get_path()).file_name() {
            Some(name) => Ok(name),
            None => Err(io::Error::new(io::ErrorKind::Other, "Invalid path ending")),
        }
    }

    pub fn get_filetype(&self) -> FileType {
        match self {
            FsEntry::File(_, _) => FileType::RegularFile,
            FsEntry::Directory(_) => FileType::Directory,
        }
    }
}

/// Will keep all the necessary info to provide real
/// data to the fuse lib
/// For now this is given to the fuse controler on creation and we do NOT have
/// ownership during the runtime.
pub struct Provider {
    pub next_inode: Ino,
    pub index: FsIndex,
    pub local_source: PathBuf,
    pub metal_handle: Dir,
    pub tx: UnboundedSender<ToNetworkMessage>,
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
