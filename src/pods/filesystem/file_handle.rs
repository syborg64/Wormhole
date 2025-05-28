use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    error::{WhError, WhResult},
    pods::arbo::{InodeId, LOCK_TIMEOUT},
};

#[derive(Debug)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
    Execute,
}

pub type UUID = u64;

#[derive(Debug)]
pub struct FileHandle {
    pub uuid: u64,
    pub perm: AccessMode,
    pub no_atime: bool,
    pub direct: bool,
}

#[derive(Debug)]
pub struct FileHandleManager {
    pub handles: HashMap<InodeId, FileHandle>,
}

pub fn new_uuid() -> UUID {
    let mut hasher = DefaultHasher::new();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("We are earlier than 1970");
    since_the_epoch.hash(&mut hasher);
    hasher.finish()
}

impl FileHandleManager {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    pub fn read_lock<'a>(
        file_handle_manager: &'a Arc<RwLock<FileHandleManager>>,
        called_from: &'a str,
    ) -> WhResult<RwLockReadGuard<'a, FileHandleManager>> {
        file_handle_manager
            .try_read_for(LOCK_TIMEOUT)
            .ok_or(WhError::WouldBlock {
                called_from: called_from.to_owned(),
            })
    }

    pub fn write_lock<'a>(
        file_handle_manager: &'a Arc<RwLock<FileHandleManager>>,
        called_from: &'a str,
    ) -> WhResult<RwLockWriteGuard<'a, FileHandleManager>> {
        file_handle_manager
            .try_write_for(LOCK_TIMEOUT)
            .ok_or(WhError::WouldBlock {
                called_from: called_from.to_owned(),
            })
    }
}
