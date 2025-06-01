use crate::pods::arbo::Arbo;
use crate::pods::network::pull_file::PullError;
use crate::{error::WhError, pods::arbo::InodeId};
use custom_error::custom_error;

use super::fs_interface::FsInterface;

custom_error! {
    /// Error describing the read syscall
    pub ReadError
    WhError{source: WhError} = "{source}",
    PullError{source: PullError} = "{source}",
    LocalReadFailed{io: std::io::Error} = "Local read failed: {io}",
    CantPull = "Unable to pull file"
}

impl FsInterface {
    pub fn read_file(
        &self,
        file: InodeId,
        offset: usize,
        buf: &mut [u8],
    ) -> Result<usize, ReadError> {
        let ok = match self.network_interface.pull_file_sync(file)? {
            None => true,
            Some(call) => self.network_interface.callbacks.n_wait_for(call)?,
        };

        if !ok {
            return Err(ReadError::CantPull);
        }

        self.disk
            .read_file(
                &Arbo::n_read_lock(&self.arbo, "read_file")?.n_get_path_from_inode_id(file)?,
                offset,
                buf,
            )
            .map_err(|io| ReadError::LocalReadFailed { io })
    }
}
