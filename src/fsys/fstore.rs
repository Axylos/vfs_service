use std::collections;
use std::ffi::{OsStr};
use crate::fsys::inode::{Inode, NodeData, FileNode};

const UID: u32 = 1000;
const GID: u32 = 1000;
pub struct FileStore {
    file_table: collections::HashMap<u64, Inode>,
    ino_ctr: u64,
}

impl FileStore {
    pub fn new() -> FileStore {
        let mut f = FileStore {
            file_table: collections::HashMap::new(),
            ino_ctr: 2,
        };


        let data = FileNode::new(); 
        let node_data = NodeData::File(data);
        let name = OsStr::new("root");
        let node = Inode::new(1, node_data, &name, UID, GID);

        f.file_table.insert(1, node);
        f
    }

    pub fn get(&self, id: &u64) -> Option<&Inode> {
        self.file_table.get(id)
    }

}

