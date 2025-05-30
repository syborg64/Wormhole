use crate::pods::{
    arbo::{Arbo, InodeId},
    filesystem::file_handle::{AccessMode, FileHandleManager},
};

use crate::error::WhError;

use custom_error::custom_error;

use super::fs_interface::FsInterface;

custom_error! {pub OpenError
    WhError{source: WhError} = "{source}",
    TruncReadOnly = "You can't truncate a file while opening in read-only",
    WrongPermissions = "Tried to open a file without permission",
    MultipleAccessFlags = "Multiple access flags given",
}

const FMODE_EXEC: i32 = 0x20;

const EXECUTE_BIT_FLAG: u16 = 1u16;
const WRITE_BIT_FLAG: u16 = 2u16;
const READ_BIT_FLAG: u16 = 4u16;

pub fn check_permissions(flags: i32, inode_perm: u16) -> Result<AccessMode, OpenError> {
    match flags & libc::O_ACCMODE {
        libc::O_RDONLY => {
            if inode_perm & READ_BIT_FLAG == 0 {
                Err(OpenError::WrongPermissions)
            //Behavior is undefined, but most filesystems return EACCES
            } else if flags & libc::O_TRUNC != 0 {
                //EACCESS
                Err(OpenError::TruncReadOnly)
            //Open is from internal exec syscall
            } else if flags & FMODE_EXEC != 0 {
                if inode_perm & EXECUTE_BIT_FLAG == 0 {
                    Err(OpenError::WrongPermissions)
                } else {
                    Ok(AccessMode::Execute)
                }
            } else {
                Ok(AccessMode::Read)
            }
        }
        libc::O_WRONLY if (inode_perm & WRITE_BIT_FLAG == 0) => Err(OpenError::WrongPermissions),
        libc::O_WRONLY => Ok(AccessMode::Write),
        libc::O_RDWR if (inode_perm & WRITE_BIT_FLAG == 0 || inode_perm & READ_BIT_FLAG == 0) => {
            Err(OpenError::WrongPermissions)
        }
        libc::O_RDWR => Ok(AccessMode::ReadWrite),
        //Exactly one access mode flag must be specified
        _ => Err(OpenError::MultipleAccessFlags),
    }
}

impl FsInterface {
    pub fn open(&self, ino: InodeId, flags: i32) -> Result<u64, OpenError> {
        let inode_perm = Arbo::n_read_lock(&self.arbo, "open")?
            .n_get_inode(ino)?
            .meta
            .perm;

        let perm = check_permissions(flags, inode_perm)?;

        if flags & libc::O_TRUNC != 0 {
            //TODO: Trunc over the network
        }

        // libc::O_CREAT is never set, The flag is set only with the create syscall

        let mut file_handles = FileHandleManager::write_lock(&self.file_handles, "open")?;
        file_handles
            .insert_new_file_handle(flags, perm)
            .map_err(|err| err.into())
    }
}
