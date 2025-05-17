use crate::{
    error::WhError,
    pods::arbo::{Arbo, InodeId},
};
use custom_error::custom_error;

use super::{
    file_handle::{AccessMode, FileHandle, FileHandleManager},
    fs_interface::FsInterface,
};

custom_error! {
    /// Error describing the removal of a [Inode] from the [Arbo]
    pub WriteError
    WhError{source: WhError} = "{source}",
    LocalWriteFailed{io: std::io::Error} = "Local write failed: {io}",
    NoFileHandle = "The file doesn't have a file handle",
    NoWritePermission = "The permissions doesn't allow to write",
    BadFd = "The file handle and the inode id doesn't match",
}

fn check_file_handle(
    file_handle: Option<&FileHandle>,
    file_handle_id: u64,
) -> Result<&FileHandle, WriteError> {
    match file_handle {
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
        let file_handle = check_file_handle(file_handles.handles.get(&id), file_handle);
        log::error!("=> {:?}, {:?}", file_handles, file_handle);
        if file_handle.is_err() {
            return Err(file_handle.err().unwrap());
        }

        let arbo = Arbo::n_read_lock(&self.arbo, "fs_interface.write")?;
        let path = arbo.n_get_path_from_inode_id(id)?;
        self.network_interface
            .update_file_size_locally(id, offset + data.len() as u64)?;

        let mut meta = arbo.n_get_inode(id)?.meta.clone();
        drop(arbo);

        let newsize = offset + data.len() as u64;
        if newsize > meta.size {
            meta.size = newsize;
            self.network_interface.n_update_metadata(id, meta)?;
        }

        let written = self
            .disk
            .write_file(path, data, offset)
            .map_err(|io| WriteError::LocalWriteFailed { io })?;

        self.network_interface.revoke_remote_hosts(id)?;
        Ok(written)
    }
}
