use crate::{
    error::WhError,
    pods::arbo::{Arbo, InodeId},
};
use custom_error::custom_error;

use super::fs_interface::FsInterface;

custom_error! {
    /// Error describing the removal of a [Inode] from the [Arbo]
    pub WriteError
    WhError{source: WhError} = "{source}",
    LocalWriteFailed{io: std::io::Error} = "Local write failed: {io}"
}

impl FsInterface {
    pub fn write(&self, id: InodeId, data: &[u8], offset: u64) -> Result<u64, WriteError> {
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
