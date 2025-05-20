use crate::pods::{
    arbo::InodeId,
    filesystem::file_handle::{new_uuid, AccessMode, FileHandle, FileHandleManager},
};

use crate::error::WhError;

use custom_error::custom_error;

use super::fs_interface::FsInterface;

custom_error! {pub OpenError
    WhError{source: WhError} = "{source}",
    TruncReadOnly = "You can't truncate a file while opening in read-only",
    MultipleAccessFlags = "Multiple access flags given",
}

const FMODE_EXEC: i32 = 0x20;

impl FsInterface {
    pub fn open(&self, id: InodeId, flags: i32) -> Result<u64, OpenError> {
        let perm = match flags & libc::O_ACCMODE {
            libc::O_RDONLY => {
                //Behavior is undefined, but most filesystems return EACCES
                if flags & libc::O_TRUNC != 0 {
                    //EACCESS
                    return Err(OpenError::TruncReadOnly);
                }
                if flags & FMODE_EXEC != 0 {
                    //Open is from internal exec syscall
                    AccessMode::Execute
                } else {
                    AccessMode::Read
                }
            }
            libc::O_WRONLY => AccessMode::Write,
            libc::O_RDWR => AccessMode::ReadWrite,
            //Exactly one access mode flag must be specified
            _ => return Err(OpenError::MultipleAccessFlags),
        };

        if flags & libc::O_TRUNC != 0 {
            //TODO: Trunc over the network
        }

        // Nothing to do, the kernel already call make_inode
        //if flags & libc::O_CREAT != 0 { }

        let direct = flags & libc::O_DIRECT != 0;
        let no_atime = flags & libc::O_NOATIME != 0;

        let uuid = new_uuid();

        let mut file_handles = FileHandleManager::write_lock(&self.file_handles, "open")?;
        file_handles.handles.insert(
            uuid,
            FileHandle {
                uuid,
                perm,
                direct,
                no_atime,
            },
        );
        Ok(uuid)
    }
}
