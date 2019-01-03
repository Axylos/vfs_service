use crate::file_tree;
use std::str;
use crate::wiki;
use fuse::{
    FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyOpen, ReplyWrite, Request, ReplyXattr, ReplyIoctl
};
use libc::ENOENT;
use log;
use std::ffi::OsStr;
use std::path;
use time::Timespec;

const EOF: u64 = 04;

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

    fn ioctl(
        &mut self, 
        _req: &Request, 
        ino: u64, 
        fh: u64, 
        flags: u32, 
        cmd: u32, 
        data: Option<&[u8]>, 
        size: u32, 
        reply: ReplyIoctl
        ) {
        log::error!("ioctl: ino={} cmd={} data={:?} size={:?}, fh={}, flags={}", ino, cmd, data, size, fh, flags);
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
        let id = self.file_tree.touch_file(&parent, name);
        let file = self.file_tree.get(&id).unwrap().data.file_data;
        let now = time::now().to_timespec();
        let ttl = now + time::Duration::hours(2);
        log::error!("got through create");
        reply.created(&ttl, &file, id, id, flags);
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
                reply.attr(&file.ttl, &file.data.file_data);
            }
            None => {
                log::error!("none found");
                reply.error(ENOENT);
            }
        }
    }

    /*
    fn opendir(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        log::error!("opendir: {}, {}", ino, flags);
    }
    */
    fn access(&mut self, _req: &Request, ino: u64, mask: u32, reply: ReplyEmpty) {
        log::error!("access: {} {}", ino, mask);
        self.file_tree.access_file(&ino);
        let _f = self.file_tree.get(&ino).unwrap();
        reply.ok();
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        log::error!("lookup: {}, {:?}", parent, name);
        // TODO: Check to see if the file exists and reply with info
        match self.file_tree.lookup_path(&parent, name) {
            Some(file) => {
                log::error!("found file: {:?}", file);
                let _data = &file.data;

                // the generation final arg needs to be the id.
                // seems similar to fh wtf
                reply.entry(&file.ttl, &file.data.file_data, file.id);
            }
            None => {
                log::error!("no file found in lookup");
                reply.error(ENOENT);
            }
        }
    }

    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        log::error!("open: {}, {}", ino, flags);
        reply.opened(ino, flags);
    }

    fn flush(&mut self, _req: &Request, ino: u64, fh: u64, lock_owner: u64, reply: ReplyEmpty) {
        log::error!("flush: {}, {}, {}", ino, fh, lock_owner);
        reply.ok();
    }
    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        log::error!("read: {}, {}, {}, size={}", ino, fh, offset, size);
        let f = self.file_tree.get(&ino).unwrap();
        let data = &f.data.content;
        // this is just a stupid way
        // to push a value into a byte slice
        // has to be a better way
        let mut v = data.to_vec();
        //disable for now seems to work on os x
        //v.push(EOF);

        let o = offset as usize;
        let d: &[u8] = &v[o..];
        reply.data(d)
    }

    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        log::error!("unlink {} {:?}", parent, name);
        self.file_tree.unlink(&parent, name);
        reply.ok();
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
        let size = self.file_tree.write(ino, data, flags, offset);
        log::error!("size={}", size);
        // must return exact same size as data that was requested to be written
        // or else stupid io invalid arg error or something happens
        // really stupid
        reply.written(w_size)
    }

    fn getxattr(
        &mut self, 
        _req: &Request, 
        ino: u64, 
        name: &OsStr, 
        size: u32, 
        reply: ReplyXattr
        ) {
        log::error!("getxattr: ino={} name={:?} size={}", ino, name, size);
        let f = self.file_tree.get(&ino).unwrap();
        log::error!("found xattr {:?}", f.xattr);
        match f.xattr.get(name) {
            Some(attr) => {
                let bytes = attr.clone().into_bytes();
                log::error!("bytes: {:?}", &bytes);

                reply.data(&bytes);
            }
            None => reply.error(61)
        
        }

    }


    fn listxattr(
        &mut self, 
        _req: &Request, 
        ino: u64, 
        size: u32, 
        reply: ReplyXattr
        ) {
        let f = self.file_tree.get(&ino).unwrap();
        let names: Vec<u8> = f.xattr.keys()
            .map(|s| s.clone().into_string()
                 .unwrap()
                 .into_bytes())
            .flatten()
            .collect();

        reply.data(&names[..]);
    } 


    fn setxattr(
        &mut self, 
        _req: &Request, 
        ino: u64, 
        name: &OsStr, 
        value: &[u8], 
        flags: u32, 
        position: u32, 
        reply: ReplyEmpty

        ) {

        log::error!("setxattr: ino={} name={:?} value={:?} flags={} position={}", ino, name, value, flags, position);

        match self.file_tree.get_mut(&ino) {
            Some(f) => {
                match str::from_utf8(value) {
                    Ok(data) => {
                        log::error!("data: {}", data);
                        f.xattr.insert(name.to_os_string(), data.to_string());

                    }
                    Err(e) => log::error!("err: {:?}", e)
                }
                log::error!("{:?}", str::from_utf8(value));
                reply.ok();
            }
            None => reply.error(ENOENT)

        }
    }

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
        let now = time::now().to_timespec();
        if let Some(_) = size {
            self.file_tree.clear_file(&ino);
        }
        let file = self.file_tree.get(&ino).unwrap();
        reply.attr(&now, &file.data.file_data);
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
}
