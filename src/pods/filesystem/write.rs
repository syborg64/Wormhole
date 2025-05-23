use crate::{
    error::WhError,
    pods::arbo::{Arbo, InodeId},
};
use custom_error::custom_error;
use parking_lot::RwLockReadGuard;

use super::{
    file_handle::{AccessMode, FileHandle, FileHandleManager},
    fs_interface::FsInterface,
};

custom_error! {
    /// Error describing the write syscall
    pub WriteError
    WhError{source: WhError} = "{source}",
    LocalWriteFailed{io: std::io::Error} = "Local write failed: {io}",
    NoFileHandle = "The file doesn't have a file handle",
    NoWritePermission = "The permissions doesn't allow to write",
    BadFd = "The file handle and the inode id doesn't match",
}

fn check_file_handle<'a>(
    file_handles: &'a RwLockReadGuard<FileHandleManager>,
    file_handle_id: u64,
) -> Result<&'a FileHandle, WriteError> {
    match file_handles.handles.get(&file_handle_id) {
        Some(&FileHandle {
            perm: AccessMode::Read,
            direct: _,
            uuid: _,
            no_atime: _,
        }) => return Err(WriteError::NoWritePermission),
        Some(&FileHandle {
            perm: AccessMode::Execute,
            direct: _,
            uuid: _,
            no_atime: _,
        }) => return Err(WriteError::NoWritePermission),
        Some(&FileHandle {
            perm: _,
            direct: _,
            uuid,
            no_atime: _,
        }) if uuid != file_handle_id => return Err(WriteError::BadFd),
        None => return Err(WriteError::NoFileHandle),
        Some(file_handle) => Ok(file_handle),
    }
}

impl FsInterface {
    pub fn write(
        &self,
        id: InodeId,
        data: &[u8],
        offset: u64,
        file_handle: u64,
    ) -> Result<u64, WriteError> {
        let file_handles = FileHandleManager::read_lock(&self.file_handles, "write")?;
        let file_handle_res = check_file_handle(&file_handles, file_handle);
        log::error!(
            "=> {:?}, {:?}, {:?}, {:?}",
            file_handles,
            file_handle_res,
            id,
            file_handle
        );
        if file_handle_res.is_err() {
            return Err(file_handle_res.err().unwrap());
        }
        log::debug!("1");

        let arbo = Arbo::n_read_lock(&self.arbo, "fs_interface.write")?;
        log::debug!("2");
        let path = arbo.n_get_path_from_inode_id(id)?;
        log::debug!("4");

        let mut meta = arbo.n_get_inode(id)?.meta.clone();
        log::debug!("5");
        drop(arbo);

        log::debug!("6");
        let newsize = offset + data.len() as u64;
        log::debug!("7");
        if newsize > meta.size {
            log::debug!("7.5");
            meta.size = newsize;
        }
        log::debug!("8");

        let written = self
            .disk
            .write_file(path, data, offset)
            .map_err(|io| WriteError::LocalWriteFailed { io })?;
        log::debug!("9");

        self.network_interface.revoke_remote_hosts(id, meta)?;
        log::debug!("10");
        Ok(written)
    }
}
