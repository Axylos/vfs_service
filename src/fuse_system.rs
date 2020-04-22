use fuse::{
    FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyOpen, ReplyWrite, Request,
};
use std::ffi::OsStr;
use time::Timespec;

use std::path;

extern crate file_store;
use file_store::fstore::FileStore;

use file_node::SingleService;

use libc::{ENOENT, ENOTDIR};

pub struct Fs {
    store: FileStore,
}

impl Fs {
    pub fn new(svcs: Vec<Box<dyn SingleService + Send>>) -> Fs {
        let mut fs = Fs {
            store: FileStore::new(),
        };

        fs.register_services(svcs);

        fs
    }

    fn register_services(&mut self, svcs: Vec<Box<dyn SingleService + Send>>) {
        self.store.register_services(svcs);
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

    fn rmdir(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
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
        reply: ReplyEmpty,
    ) {
        match self.store.rename(&parent, name, newparent, newname) {
            Ok(_) => reply.ok(),
            Err(_) => reply.error(ENOENT),
        }
    }

    fn mkdir(&mut self, _req: &Request, parent: u64, name: &OsStr, mode: u32, reply: ReplyEntry) {
        log::info!("creating a dir");

        let node = self.store.create_dir(parent, name, mode);
        let ttl = Timespec::new(1, 0);
        match node {
            Some(dir) => {
                reply.entry(&ttl, &dir.attr, dir.id);
            }
            _ => reply.error(ENOENT),
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
                let ttl = Timespec::new(1, 0);
                log::error!("got through create");
                reply.created(&ttl, &file, id, id, flags);
            }
            None => {
                log::error!("not a valid parent");
                reply.error(ENOTDIR);
            }
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        reply: ReplyData,
    ) {
        match self.store.read_file(&ino) {
            Some(data) => {
                // need to also restrict length of response by size bytes
                let d = &data[offset as usize..];

                reply.data(&d)
            }
            None => reply.error(ENOENT),
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
        match self.store.read_dir_children(&ino) {
            Some(children) => {
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
            None => reply.error(ENOENT),
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
        atime: Option<Timespec>,
        mtime: Option<Timespec>,
        fh: Option<u64>,
        crtime: Option<Timespec>,
        chgtime: Option<Timespec>,
        bkuptime: Option<Timespec>,
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
        let now = Timespec::new(1, 0);
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
                let ttl = Timespec::new(1, 0);
                log::info!("found filez: {:?} {:?}", file.attr, ttl);
                reply.attr(&ttl, &file.attr);
            }
            None => {
                log::error!("none found! {:?}", ino,);
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
