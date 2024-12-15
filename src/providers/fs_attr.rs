use fuser::FileType;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type Permission = u16;

pub type ATime = SystemTime;
pub type MTime = SystemTime;
pub type CTime = SystemTime;
pub type CrTime = SystemTime;

pub type Kind = FileType;

pub type Bytes = u64;
pub type Blocks = u64;
pub type Blksize = u32;

pub type Uid = u32;
pub type Gid = u32;

pub type Nlink = u32;
pub type Rdev = u32; //NOTE - Device ID - Only used for FileType::BlockDevice or FileType::CharDevice otherwise use 0
pub type Flags = u32;

pub struct FsAttr {
    pub fs_time: (ATime, MTime, CTime, CrTime),
    pub kind: Kind,
    pub permission: Permission,
    pub size: (Bytes, Blocks, Blksize),
    pub id: (Uid, Gid),
    pub nlink: Nlink,
    pub rdev: Rdev,
    pub flags: Flags,
}

impl FsAttr {
    pub fn get_size_in_bytes(&self) -> &Bytes {
        &self.size.0
    }

    pub fn get_size_in_blocks(&self) -> &Blocks {
        &self.size.1
    }

    pub fn get_blocks_size(&self) -> &Blksize {
        &self.size.2
    }

    pub fn get_last_access(&self) -> &ATime {
        &self.fs_time.0
    }

    pub fn get_last_modif(&self) -> &MTime {
        &self.fs_time.1
    }

    pub fn get_last_change(&self) -> &CTime {
        &self.fs_time.2
    }

    pub fn get_creation_time(&self) -> &CrTime {
        &self.fs_time.3
    }

    pub fn get_kind(&self) -> &Kind {
        &self.kind
    }

    pub fn get_permission(&self) -> &Permission {
        &self.permission
    }

    pub fn get_uid(&self) -> &Uid {
        &self.id.0
    }

    pub fn get_gid(&self) -> &Gid {
        &self.id.1
    }

    pub fn get_hard_link_num(&self) -> &Nlink {
        &self.nlink
    }

    pub fn get_device_id(&self) -> &Rdev {
        &self.rdev
    }

    pub fn get_flags(&self) -> &Flags {
        &self.flags
    }
}
