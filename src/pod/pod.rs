use fuser::BackgroundSession;
use crate::fuse::start::mount_fuse;

/// Struct pod contain:
/// - session: a value used for talk with the thread of the filesystem
/// - mountpoint: the path of the mount point of the pod
/// - active_fh: a vector containing all fh of the mount points
pub struct Pod {
    session: BackgroundSession,
    pub mountpoint: String,
    active_fh: Vec<u64>,
    // TODO - more variables on network
}

impl Pod {
    pub fn new(mountpoint: String) -> Self {
        Pod {
            session: mount_fuse(&mountpoint),
            mountpoint,
            active_fh: Vec::new(),
        }
    }

    // NOTE - Objective : properly close the mounted folder / network
    // now automatic when "session" is dropped
    // Made for when the instance exits, not for a definitive exit of the network
    // pub fn unmont() -> Result<(), Error> {
    //     Ok(())
    // }
}
