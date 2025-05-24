use custom_error::custom_error;

use crate::pods::arbo::Arbo;
use crate::{error::WhError, pods::arbo::InodeId};

use super::fs_interface::FsInterface;

custom_error! {
    /// Error describing the read syscall
    pub ReadError
    WhError{source: WhError} = "{source}",
    CantPull = "Unable to pull file"
}

impl FsInterface {
    pub fn n_read_file(&self, file: InodeId, offset: u64, len: u64) -> Result<Vec<u8>, ReadError> {
        // let cb = self.network_interface.pull_file_sync(file)?;

        // let status = match cb {
        //     None => true,
        //     Some(call) => self.network_interface.callbacks.wait_for(call)?,
        // };

        // if !status {
        //     return Err(ReadError::CantPull);
        // }

        // self.disk.read_file(
        //     Arbo::n_read_lock(&self.arbo, "read_file")?.n_get_path_from_inode_id(file)?,
        //     offset,
        //     len,
        // )
        return Ok(vec![]);
    }
}
