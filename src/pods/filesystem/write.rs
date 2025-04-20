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
    pub fn write(&self, id: InodeId, data: &[u8], offset: usize) -> Result<usize, WriteError> {
        let written = {
            let arbo = Arbo::n_read_lock(&self.arbo, "fs_interface.write")?;
            let path = arbo.n_get_path_from_inode_id(id)?;
            let mut meta = arbo.n_get_inode(id)?.meta.clone();
            drop(arbo);

            let newsize = offset + data.len();
            if newsize as u64 > meta.size {
                meta.size = newsize as u64;
                self.network_interface.n_update_metadata(id, meta)?;
            }
            self.disk
                .write_file(&path, data, offset)
                .map_err(|io| WriteError::LocalWriteFailed { io })?
        };

        self.network_interface.revoke_remote_hosts(id)?;
        Ok(written)
    }
}
