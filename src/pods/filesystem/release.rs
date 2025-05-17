use crate::error::WhResult;

use super::{file_handle::FileHandleManager, fs_interface::FsInterface};

impl FsInterface {
    pub fn release(&self, ino: u64) -> WhResult<()> {
        let mut file_handles = FileHandleManager::write_lock(&self.file_handles, "write")?;
        file_handles.handles.remove(&ino);
        return Ok(());
    }
}
