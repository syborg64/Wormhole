use std::{collections::HashMap, path::Path};

use crate::fuse::start::mount_fuse;
use fuser::BackgroundSession;
use walkdir::WalkDir;

use super::COPIED_ROOT;

pub type fs_index = HashMap<u64, (fuser::FileType, String)>;

pub struct Pod {
    session: BackgroundSession, // Fuse handle
    pub index: fs_index,        // Total fs indexing
    pub mountpoint: String,     // mountpoint of the fuse fs
                                //active_fh: Vec<u64>,
                                // TODO - more variables on network
}

impl Pod {
    pub fn new(mountpoint: String) -> Self {
        let index = Self::index_folder(&Path::new(&mountpoint));
        println!("starting arbo is {:?}", index);
        Pod {
            session: mount_fuse(&mountpoint),
            index,
            mountpoint,
            //active_fh: Vec::new(),
        }
    }

    fn index_folder(pth: &Path) -> fs_index {
        let mut arbo: fs_index = HashMap::new();
        let mut inode: u64 = 2;

        arbo.insert(1, (fuser::FileType::Directory, COPIED_ROOT.to_owned()));

        for entry in WalkDir::new(COPIED_ROOT).into_iter().filter_map(|e| e.ok()) {
            let strpath = entry.path().display().to_string();
            let path_type = if entry.file_type().is_dir() {
                fuser::FileType::Directory
            } else if entry.file_type().is_file() {
                fuser::FileType::RegularFile
            } else {
                fuser::FileType::CharDevice // random to detect unsupported
            };
            if strpath != COPIED_ROOT && path_type != fuser::FileType::CharDevice {
                println!("indexing {}", strpath);
                arbo.insert(inode, (path_type, strpath));
                inode += 1;
            } else {
                println!("ignoring {}", strpath);
            }
        }
        arbo
    }
}
