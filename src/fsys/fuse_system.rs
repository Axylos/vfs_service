use std::ffi::{OsStr};
use std::time::{SystemTime, Duration};
use fuse::{
    FileType,
    Filesystem,
    ReplyAttr,
    ReplyCreate,
    ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyOpen, ReplyWrite,
    Request,
};

use std::path;

use crate::fsys::fstore::{FileStore};
use crate::fsys::inode::{NodeData};

use libc::{ENOENT, ENOTDIR};

pub struct Fs {
    store: FileStore
}

impl Fs {
    pub fn new() -> Fs {
        Fs {
            store: FileStore::new()
        }
    }
}

impl Filesystem for Fs {
    fn init(&mut self, _req: &Request) -> Result<(), i32> {
        log::info!("up and running");

        Ok(())
    }

    fn access(&mut self, _req: &Request, ino: u64, mask: u32, reply: ReplyEmpty) {
        log::error!("access: {} {}", ino, mask);
                reply.ok();
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        log::error!("called lookup");
        match self.store.lookup_path(&parent, name) {
            Some(file) => {
                log::error!("found file: {:?}", file);
                let _data = &file.data;

                // the generation final arg needs to be the id.
                // seems similar to fh wtf
                reply.entry(&file.ttl, &file.attr, file.id);
            }
            None => {
                log::error!("no file found in lookup: {:?} {:?}", name, parent);
                reply.error(ENOENT);
            }
        }

    }

    fn rmdir(
        &mut self, 
        _req: &Request, 
        parent: u64, 
        name: &OsStr, 
        reply: ReplyEmpty

        ) {
        log::error!("unlink {} {:?}", parent, name);
        self.store.unlink(&parent, name);
        reply.ok();
    }

    fn rename(
        &mut self, 
        _req: &Request, 
        parent: u64, 
        name: &OsStr, 
        newparent: u64, 
        newname: &OsStr, 
        reply: ReplyEmpty
        ) {
        self.store.rename(&parent, name, newparent, newname);
        reply.ok();
    }


    fn mkdir(
        &mut self, 
        _req: &Request, 
        parent: u64, 
        name: &OsStr, 
        mode: u32, 
        reply: ReplyEntry
        ) {
        log::info!("creating a dir");

        let node = self.store.create_dir(parent, name, mode);
        let ttl = Duration::from_secs(1);
        match node {
            Some(dir) => {
                reply.entry(&ttl, &dir.attr, dir.id);
            }
            _ => reply.error(ENOENT)
        }
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
        let id = self.store.touch_file(&parent, name);
        match self.store.get(&id) {
            Some(f) => {
                let file = f.attr;
                let ttl = Duration::from_secs(1);
                log::error!("got through create");
                reply.created(&ttl, &file, id, id, flags);
            }
            None => {
                log::error!("not a valid parent");
                reply.error(ENOTDIR);
            }
        }
    }


    fn read(&mut self, _req: &Request, ino: u64, fh: u64, offset: i64, _size: u32, reply: ReplyData) {
        match self.store.get(&ino) {
            Some(f) => {
                match &f.data {
                    NodeData::File(file) => {
                        let data = &file.content;
                        // this is just a stupid way
                        // to push a value into a byte slice
                        // has to be a better way
                        let v = data.to_vec();
                        //disable for now seems to work on os x
                        //v.push(EOF);

                        let o = offset as usize;
                        let d: &[u8] = &v[o..];
                        reply.data(d)
                    }
                    _ => reply.error(ENOENT)
                }
            },
            None => {
                log::error!("read failed {:?} {:?}", ino, fh);
reply.error(ENOENT)
            }
        }
    }

    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        flags: u32,
        reply: ReplyWrite,
    ) {
        log::error!("write: {} {} {} {:?} {}", ino, fh, offset, data, flags);
        let w_size = std::mem::size_of_val(data) as u32;
        log::error!("write size: {}", w_size as u32);
        let size = self.store.write(ino, data, flags, offset);
        log::error!("size={}", size);
        // must return exact same size as data that was requested to be written
        // or else stupid io invalid arg error or something happens
        // really stupid
        reply.written(w_size)
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
        match self.store.get(&ino) {
            Some(inode) => {
                match &inode.data {
                    NodeData::RegularDir(node) => {
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
                                match self.store.get(&id) {
                                    Some(f) => {
                                        reply.add(f.id, ctr, f.attr.kind, &f.path);
                                        ctr += 1;
                                        log::error!("{:?}", f);
                                    }
                                    None => log::error!(
                                        "dangling child reference: parent={} child={}",
                                        &ino,
                                        &id
                                        ),
                                }
                            }
                        }
                        reply.ok();

                    }
                    NodeData::ServiceDir(node) => {
                        let children = &node.children;
                        log::error!("service children: {:?}", children);
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
                                match self.store.get(&id) {
                                    Some(f) => {
                                        reply.add(f.id, ctr, f.attr.kind, &f.path);
                                        ctr += 1;
                                        log::error!("{:?}", f);
                                    }
                                    None => log::error!(
                                        "dangling child reference: parent={} child={}",
                                        &ino,
                                        &id
                                        ),
                                }
                            }
                        }
                        reply.ok();
                    }
                    NodeData::File(_node) => {
                        log::error!("file found:");
                        reply.error(ENOENT);
                    }
                }
            }
            None => {
                reply.error(ENOENT);
            }
        }
    }

    /*
    fn getlk(
        &mut self, 
        _req: &Request, 
        _ino: u64, 
        _fh: u64, 
        _lock_owner: u64, 
        _start: u64, 
        _end: u64, 
        _typ: u32, 
        _pid: u32, 
        _reply: ReplyLock
        ) {
        log::error!("getlk!");
    }
    */

    fn setattr(
        &mut self,
        _req: &Request,
        ino: u64,
        mode: Option<u32>,
        gid: Option<u32>,
        uid: Option<u32>,
        size: Option<u64>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        fh: Option<u64>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        log::error!(
            "set attr: ino={} mode={:?} gid={:?} \
             uid={:?} size={:?} atime{:?} \
             mtime={:?} fh={:?} crtime={:?} \
             chgtime={:?} bkuptime={:?} flags={:?}",
            ino,
            mode,
            gid,
            uid,
            size,
            atime,
            mtime,
            fh,
            crtime,
            chgtime,
            bkuptime,
            flags
            );
        let now = Duration::from_secs(1);
        if let Some(_) = size {
            self.store.clear_file(&ino);
        }
        let file = self.store.get(&ino).unwrap();
        reply.attr(&now, &file.attr);

    }

    fn flush(&mut self, _req: &Request, ino: u64, fh: u64, lock_owner: u64, reply: ReplyEmpty) {
        log::error!("flush: {}, {}, {}", ino, fh, lock_owner);
        reply.ok();

    }

    fn release(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        flags: u32,
        lock_owner: u64,
        flush: bool,
        reply: ReplyEmpty,

        ) {
        log::error!("release {} {} {} {} {}", ino, fh, flags, lock_owner, flush);
        reply.ok();
    }

    /*
    fn opendir(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        log::error!("opendir: {}, {}", ino, flags);
    }
    */
    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        log::error!("open called {:?} {:?}", ino, flags);
        reply.opened(ino, flags);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.store.get(&ino) {
            Some(file) => {
                let ttl = Duration::from_secs(1);
                log::info!("found filez: {:?} {:?}", file.attr, ttl);
                reply.attr(&ttl, &file.attr);
            }
            None => {
                log::error!("none found! {:?}", ino, );
                reply.error(ENOENT);
            }
        }

    }

    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        log::error!("unlink {} {:?}", parent, name);
        self.store.unlink(&parent, name);
        reply.ok();
    }

}
