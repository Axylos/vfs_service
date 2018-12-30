use log;
use crate::file_tree;
use fuse::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyDirectory, Request, ReplyEntry, ReplyOpen};
use libc::{ENOENT, ENOSYS};
use std::ffi::OsStr;
use std::path::Path;
use time::Timespec;

pub struct Fs {
    file_tree: file_tree::FileMap,
}

impl Fs {
    pub fn new() -> Fs {
        Fs {
            file_tree: file_tree::FileMap::new(),
        }
    }
}

impl Filesystem for Fs {
    fn init(&mut self, _req: &Request) -> Result<(), i32> {
        log::trace!("up and running");

        Ok(())
    }

    fn create(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        flags: u32,
        reply: ReplyCreate,
    ) {
        let now = time::now().to_timespec();
        log::trace!("create: {}, {:?}, {}, {}", parent, name, mode, flags);
        let n = name.to_str().unwrap().clone();
        let id = self.file_tree.touch_file(&parent, String::from(n));
        let file = self.file_tree.get(&id).unwrap().data.file_data;
        let now = time::now().to_timespec();
        log::trace!("got through create");
        reply.created(&now, &file, 1, 1, 2);
    }
    fn readdir(
        &mut self,
        req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        log::trace!("readdir: {}, {}, {}, {:?}", ino, fh, offset, req);
        match self.file_tree.get(&ino) {
            Some(node) => {
                let children = &node.children;
                if !children.is_empty() {
                    let child = self.file_tree.get(children[0]);
                }
                let child = 
                if offset == 0 {
                    reply.add(1, 0, FileType::Directory, &Path::new("."));
                    reply.add(1, 1, FileType::Directory, &Path::new(".."));
                    reply.ok();
                } else if offset == 1 {
                    reply.ok();
                }
            }
            None => reply.error(ENOENT),
        };
    }

    fn getattr(&mut self, req: &Request, ino: u64, reply: ReplyAttr) {
        log::trace!("getattr: {}, {:?}", ino, req);
        match self.file_tree.get(&ino) {
            Some(file) => {
                let ttl = Timespec::new(1, 0);
                let ts = Timespec::new(0, 0);
                reply.attr(&ttl, &file.data.file_data);
            }
            None => {
                log::trace!("none found");
                reply.error(ENOENT);
            }
        }
    }

 fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {  
     log::trace!("lookup: {}, {:?}", parent, name);
     // TODO: Check to see if the file exists and reply with info
     match self.file_tree.lookup_path(&parent, name) {
         Some(file) => {
             log::trace!("found file: {:?}", file);
             let now = time::now().to_timespec();
             reply.entry(&now, &file.data.file_data, 2);
         },
         None => reply.error(ENOENT)
     }
 }

 fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
     log::trace!("open: {}, {}", ino, flags);
     reply.opened(ino, flags);

 }

 fn setattr(&mut self, _req: &Request, ino: u64, _mode: Option<u32>, _gid: Option<u32>, _uid: Option<u32>, _size: Option<u64>, _atime: Option<Timespec>, _mtime: Option<Timespec>, _fh: Option<u64>, _crtime: Option<Timespec>, _chgtime: Option<Timespec>, _bkuptime: Option<Timespec>, _flags: Option<u32>, reply: ReplyAttr) {
     log::trace!("{}", ino);
     let file = self.file_tree.get(&ino).unwrap();
     let now = time::now().to_timespec();
     reply.attr(&now, &file.data.file_data);
 }
}
