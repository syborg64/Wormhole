use std::{collections::HashMap, path::Path};

use crate::fuse::start::mount_fuse;
use fuser::BackgroundSession;
use walkdir::WalkDir;

use super::COPIED_ROOT;

pub struct Pod {
    session: BackgroundSession, // Fuse handle
    pub index: HashMap<u64, String>, // Total fs indexing
    pub mountpoint: String, // mountpoint of the fuse fs
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

    fn index_folder(pth: &Path) -> HashMap<u64, String> {
        let mut arbo: HashMap<u64, String> = HashMap::new();
        let mut inode: u64 = 2;

        arbo.insert(1, pth.to_string_lossy().to_string());

        for entry in WalkDir::new(COPIED_ROOT).into_iter().filter_map(|e| e.ok()) {
            let strpath = entry.path().display().to_string();
            if strpath != COPIED_ROOT {
                println!("indexing {}", strpath);
                arbo.insert(inode, strpath);
                inode += 1;
            } else {
                println!("ignoring {}", strpath);
            }
        }
        arbo
    }
}
