use crate::{
    error::WhResult,
    pods::{
        arbo::InodeId,
        filesystem::file_handle::{new_unique_handle, FileHandleManager},
    },
};

use super::fs_interface::FsInterface;

impl FsInterface {
    pub fn open(&self, id: InodeId) -> WhResult<()> {
        let handle = new_unique_handle();
        log::debug!("Handle: {handle}");

        let mut file_handles = FileHandleManager::write_lock(&self.file_handles, "open")?;
        file_handles.handles.insert(id, handle);
        Ok(())
    }
}
