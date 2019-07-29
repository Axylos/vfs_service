use std::collections;
use std::path;
use std::ffi::{OsStr, OsString};
use fuse::{FileAttr, FileType};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct FileNode {
    pub content: Vec<u8>,
}

impl FileNode {
    pub fn new() -> FileNode {
        FileNode {
            content: Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegularDirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}

pub struct BundleServiceDirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}

pub struct ServiceDirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}

pub trait SingleService {
    fn fetch_data(query: Option<&str>) -> Result<(u64), Box<dyn std::error::Error>>;
}

pub struct NumSvc {
    pub data: u64
}

impl SingleService for NumSvc {
    fn fetch_data(query: Option<&str>) -> Result<(u64), Box<dyn std::error::Error>> { 
        match query {
            Some(q) => {
                let len = q.len() as u64;
                Ok(len)
            }
            None => Ok(0)
        }
    }
}

impl ServiceDirNode {
    pub fn new(svc: impl SingleService) -> ServiceDirNode {
        ServiceDirNode {
            children: collections::BTreeSet::new(),
            name_map: collections::HashMap::new(),
        }
    }

}


impl DirNode for ServiceDirNode {
    fn remove(&mut self, id: &u64, name: &OsStr) {
        self.children.remove(id);
        self.name_map.remove(name);
    }

    fn add(&mut self, id: u64, name: std::ffi::OsString) {
        self.children.insert(id);
        self.name_map.insert(name, id);
    }
}

impl RegularDirNode {
    pub fn new() -> RegularDirNode {
        RegularDirNode {
            children: collections::BTreeSet::new(),
            name_map: collections::HashMap::new(),

        }
    }

    pub fn get_stuff(&self) -> u64 { 4 }
}


impl DirNode for RegularDirNode {
    fn remove(&mut self, id: &u64, name: &OsStr) {
        self.children.remove(id);
        self.name_map.remove(name);
    }

    fn add(&mut self, id: u64, name: std::ffi::OsString) {
        self.children.insert(id);
        self.name_map.insert(name, id);
    }

}

pub trait DirNode {
    fn remove(&mut self, id: &u64, name: &OsStr);
    fn add(&mut self, id: u64, name: std::ffi::OsString);
}

#[derive(Debug)]
pub enum NodeData {
    File(FileNode),
    RegularDir(RegularDirNode),
}


#[derive(Debug)]
pub struct Inode {
    pub id: u64,
    pub ttl: std::time::Duration,
    pub data: NodeData,
    pub attr: FileAttr,
    pub xattr: collections::HashMap<OsString, String>,
    pub path: path::PathBuf,
}

impl Inode {
    pub fn new(id: u64, data: NodeData, name: &OsStr, _uid: u32, _gid: u32) -> Inode {
        let ttl = Duration::from_secs(1);
        let path = path::PathBuf::from(name);
        let kind = match data {
            NodeData::File(_) => FileType::RegularFile,
            NodeData::RegularDir(_) => FileType::Directory
        };
        let mut attr = build_dummy_file(kind);
        attr.uid = 501;
        attr.gid = 20;
        Inode {
            id,
            attr,
            path,
            data,
            ttl,
            xattr: collections::HashMap::new(),
        }

    }


    pub fn access(&mut self) {
        let now = SystemTime::now();
        self.attr.atime = now;
    }

}

fn build_dummy_file(kind: FileType) -> FileAttr {
    let ts = std::time::UNIX_EPOCH;
    let ino = 1;
    FileAttr {
        ino,
        kind,
        nlink: 2,
        perm: 0o755,
        rdev: 0,
        size: 0,
        atime: ts,
        ctime: ts,
        crtime: ts,
        mtime: ts,
        blocks: 0,
        flags: 0,
        gid: 0,
        uid: 0,
    }
}

