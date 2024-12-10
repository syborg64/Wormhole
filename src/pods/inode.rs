use std::{collections::HashMap, io, sync::Arc};

use crate::providers::{whpath::WhPath, FsEntry, InodeIndex};

const ROOT: InodeIndex = 0;

struct Inode {
    parent_index: Arc<InodeIndex>,
    index: Arc<InodeIndex>,
    name: String,
    entry: FsEntry,
}
struct Arbo {
    tree: Inode /* ROOT */,
    index: ArboIndex,
}

type ArboIndex = HashMap<InodeIndex, Arc<Inode>>;


impl Arbo {
    pub fn path_from_inode_index(&self, inode_index: InodeIndex) -> io::Result<WhPath> {
        if inode_index == ROOT {
            return Ok(WhPath::new("/"))
        }
        let inode = match self.index.get(&inode_index) {
            Some(inode) => inode,
            None => {
                return Err(io::Error::new(io::ErrorKind::NotFound, "entry not found"));
            }
        };

        let mut parent_path = self.path_from_inode_index(*inode.parent_index)?;
        parent_path.join(inode.name.clone());
        Ok(parent_path)
    }
}