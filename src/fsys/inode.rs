use std::collections;
use std::path;
use std::ffi::{OsStr, OsString};
use fuse::{FileAttr, FileType};
use std::time::{Duration, SystemTime};

use std::fmt;
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

#[derive(Debug, Clone)]
pub struct BundleServiceDirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}

pub trait SingleService {
    fn fetch_data(&self, query: Option<&str>) -> Vec<String>;
}

#[derive(Debug)]
pub struct Service {
    pub svc: Box<dyn SingleService>,
}

impl std::fmt::Debug for SingleService + 'static {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "it worked")
    }

}

#[derive(Debug)]
pub struct ServiceDirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
    pub service: Service
}

impl ServiceDirNode {

    fn get_data(&self, query: Option<&str>) -> Vec<String> {
        self.service.svc.fetch_data(query)
    }

    fn remove(&mut self, id: &u64, name: &OsStr) {
        self.children.remove(id);
        self.name_map.remove(name);
    }

    fn add(&mut self, id: u64, name: std::ffi::OsString) {
        self.children.insert(id);
        self.name_map.insert(name, id);
    }
}


pub struct NumSvc {
    pub data: u64
}

impl SingleService for NumSvc {
    fn fetch_data(&self, query: Option<&str>) -> Vec<String> { 
        match query {
            Some(q) => {
                let len = q.len() as u64;
                vec!("foo".to_string())
            }
            None => vec!("foo".to_string())
        }
    }
}


impl RegularDirNode {
    pub fn new() -> RegularDirNode {
        RegularDirNode {
            children: collections::BTreeSet::new(),
            name_map: collections::HashMap::new(),

        }
    }
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
    ServiceDir(ServiceDirNode)
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
            NodeData::RegularDir(_) => FileType::Directory,
            NodeData::ServiceDir(_) => FileType::Directory,
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

