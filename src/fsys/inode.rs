use std::collections;
use std::path;
use time::Timespec;
use fuse::{FileAttr};
use std::ffi::OsStr;
use std::ffi::OsString;


#[derive(Debug)]
pub struct NodeData {
    pub file_data: FileAttr,
    pub content: Vec<u8>,
}


#[derive(Debug)]
pub struct Inode {
    pub id: u64,
    pub ttl: Timespec,
    pub data: NodeData,
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
    pub xattr: collections::HashMap<OsString, String>,
    pub path: path::PathBuf,
}

impl Inode {
    pub fn access(&mut self) {
        let now = time::now().to_timespec();
        self.data.file_data.atime = now;
    }

    pub fn new(id: u64, data: NodeData, name: &OsStr) -> Inode {
        let ttl = time::now().to_timespec() + time::Duration::hours(10);
        let path = path::PathBuf::from(name);
        Inode {
            id,
            path,
            data,
            ttl,
            xattr: collections::HashMap::new(),
            children: collections::BTreeSet::new(),
            name_map: collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, id: u64, name: std::ffi::OsString) {
        self.children.insert(id);
        self.name_map.insert(name, id);
    }

    #[cfg(test)]
    fn inc(&mut self) {
        self.data.file_data.ino += 1;
    }
}

