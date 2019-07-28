use std::collections;
use std::path;
use std::ffi::{OsStr, OsString};
use fuse::{FileAttr, FileType};
use std::time::{Duration, UNIX_EPOCH, SystemTime};

const USER_DIR: u32 = 0x755;

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
pub struct DirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}

impl DirNode {
    pub fn new() -> DirNode {
        DirNode {
            children: collections::BTreeSet::new(),
            name_map: collections::HashMap::new(),

        }
    }

    pub fn remove(&mut self, id: &u64, name: &OsString) {
        self.children.remove(id);
        self.name_map.remove(name);
    }

    pub fn add(&mut self, id: u64, name: std::ffi::OsString) {
        self.children.insert(id);
        self.name_map.insert(name, id);
    }

}

#[derive(Debug, Clone)]
pub enum NodeData {
    File(FileNode),
    Dir(DirNode),
}


#[derive(Debug, Clone)]
pub struct Inode {
    pub id: u64,
    pub ttl: std::time::Duration,
    pub data: NodeData,
    pub attr: FileAttr,
    pub xattr: collections::HashMap<OsString, String>,
    pub path: path::PathBuf,
}

impl Inode {
    pub fn new(id: u64, data: NodeData, name: &OsStr, uid: u32, gid: u32) -> Inode {
        let ttl = Duration::from_secs(1);
        let path = path::PathBuf::from(name);
        let kind = match data {
            NodeData::File(_) => FileType::RegularFile,
            NodeData::Dir(_) => FileType::Directory
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

