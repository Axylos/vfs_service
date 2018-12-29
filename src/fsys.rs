use crate::file_tree;
use fuse::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyDirectory, Request, ReplyEntry};
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
        println!("up and running");

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
        println!("create: {}, {:?}, {}, {}", parent, name, mode, flags);
        let n = name.to_str().unwrap().clone();
        self.file_tree.touch_file(&parent, String::from(n));
    }
    fn readdir(
        &mut self,
        req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir: {}, {}, {}, {:?}", ino, fh, offset, req);
        match self.file_tree.get(&ino) {
            Some(node) => {
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
        println!("getattr: {}, {:?}", ino, req);
        match self.file_tree.get(&ino) {
            Some(file) => {
                let ttl = Timespec::new(1, 0);
                let ts = Timespec::new(0, 0);
                reply.attr(&ttl, &file.data.file_data);
            }
            None => {
                println!("none found");
                reply.error(ENOENT);
            }
        }
    }

 fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {  
     println!("lookup: {}, {:?}", parent, name);
     // TODO: Check to see if the file exists and reply with info
     reply.error(ENOENT);
 }
}
