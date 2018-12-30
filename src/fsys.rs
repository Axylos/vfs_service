use crate::file_tree;
use fuse::{
    FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyDirectory, ReplyEmpty, ReplyEntry,
    ReplyOpen, Request,
};
use libc::ENOENT;
use log;
use std::ffi::OsStr;
use std::path;
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
        log::error!("up and running");

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
        let _now = time::now().to_timespec();
        log::error!("create: {}, {:?}, {}, {}", parent, name, mode, flags);
        let n = name.to_str().unwrap().to_string();
        let id = self.file_tree.touch_file(&parent, n);
        let file = self.file_tree.get(&id).unwrap().data.file_data;
        let now = time::now().to_timespec();
        log::error!("got through create");
        reply.created(&now, &file, 1, 1, 2);
    }
    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        log::error!("readdir: {}, {}, {}", ino, fh, offset);
        match self.file_tree.get(&ino) {
            Some(node) => {
                let children = &node.children;
                let mut idx: u64 = 0;
                let offset = offset as u64;
                if offset > 2 {
                    idx = offset - 2;
                }

                let len = children.len() as u64;
                if offset < len + 1 as u64 {
                    reply.add(1, 0, FileType::Directory, &path::Path::new("."));
                    reply.add(1, 1, FileType::Directory, &path::Path::new(".."));
                    let mut ctr = 2 + offset as i64;
                    for id in children.range(idx..) {
                        let f = self.file_tree.get(&id).unwrap();
                        reply.add(f.id, ctr, f.data.file_data.kind, &f.path);
                        ctr += 1;
                        log::error!("{:?}", f);
                    }
                }
                reply.ok();
            }
            None => reply.error(ENOENT),
        };
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        log::error!("getattr: {}", ino);
        match self.file_tree.get(&ino) {
            Some(file) => {
                let ttl = Timespec::new(1, 0);
                //let ts = Timespec::new(0, 0);
                reply.attr(&ttl, &file.data.file_data);
            }
            None => {
                log::error!("none found");
                reply.error(ENOENT);
            }
        }
    }

    fn access(&mut self, _req: &Request, ino: u64, mask: u32, reply: ReplyEmpty) {
        log::error!("access: {} {}", ino, mask);
        reply.ok();
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        log::error!("lookup: {}, {:?}", parent, name);
        // TODO: Check to see if the file exists and reply with info
        match self.file_tree.lookup_path(&parent, name) {
            Some(file) => {
                log::error!("found file: {:?}", file);
                let now = time::now().to_timespec();
                reply.entry(&now, &file.data.file_data, 2);
            }
            None => reply.error(ENOENT),
        }
    }

    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        log::error!("open: {}, {}", ino, flags);
        reply.opened(ino, flags);
    }

    fn setattr(
        &mut self,
        _req: &Request,
        ino: u64,
        _mode: Option<u32>,
        _gid: Option<u32>,
        _uid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<Timespec>,
        _mtime: Option<Timespec>,
        _fh: Option<u64>,
        _crtime: Option<Timespec>,
        _chgtime: Option<Timespec>,
        _bkuptime: Option<Timespec>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        log::error!("{}", ino);
        let file = self.file_tree.get(&ino).unwrap();
        let now = time::now().to_timespec();
        reply.attr(&now, &file.data.file_data);
    }
}
