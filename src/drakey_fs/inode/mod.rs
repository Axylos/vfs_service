use std::collections;
use std::path;
use time;
use std::ffi;
use fuse::{FileAttr, FileType::Directory};
#[derive(Debug)]
pub struct Inode {
    pub id: u64,
    pub file_data: FileAttr,
    pub ttl: time::Timespec,
//    pub data: NodeData,
    pub xattr: collections::HashMap<ffi::OsString, String>,
    pub path: path::PathBuf,
}


impl Inode {
    pub fn new_dir(id: u64, name: &ffi::OsStr) -> Inode {
        let ts = time::now().to_timespec();
        let ino = 1;
        let attrs = FileAttr {
            ino,
            kind: Directory,
            nlink: 0,
            perm: 0o777,
            rdev: 0,
            size: 0,
            atime: ts,
            ctime: ts,
            crtime: ts,
            mtime: ts,
            blocks: 100,
            flags: 0,
            gid: 0,
            uid: 501,
        };


        let ttl = time::now().to_timespec() + time::Duration::hours(10);
        let path = path::PathBuf::from(name);
        Inode {
            id,
            path,
            file_data: attrs,
            ttl,
            xattr: collections::HashMap::new(),
        }
    }
}

