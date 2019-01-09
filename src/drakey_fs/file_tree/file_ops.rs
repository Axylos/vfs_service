use super::FileTree;
use crate::drakey_fs::inode::{node_data::NodeData, node_types};
use fuse;
use libc::ENOENT;
use time;

impl FileTree {
    pub fn get_file_attrs(&self, ino: &u64) -> Option<(&time::Timespec, &fuse::FileAttr)> {
        let f = self.files.get(ino)?;
        Some((&f.ttl, &f.file_data))
    }

    pub fn access_file(&mut self, ino: &u64) -> Result<(), i32> {
        if self.files.contains_key(ino) {
            self.files.entry(*ino).and_modify(|f| {
                f.access();
            });
            Ok(())
        } else {
            Err(ENOENT)
        }
    }

    pub fn get_file(&mut self, ino: &u64) -> Option<&mut Box<node_types::DrakeyFile + Send>> {
        let node = self.files.get_mut(ino)?;
        match &mut node.data {
            NodeData::File(f) => Some(f),
            _ => None,
        }
    }

    pub fn write_file(&mut self, ino: u64, data: &[u8], flags: u32, offset: i64) -> Option<u32> {
        log::error!("write file: ino={}", ino);
        let str = String::from_utf8_lossy(data).trim().to_string();

        let size = std::mem::size_of_val(&str.as_bytes());
        log::error!("size={:?}", size);
        log::error!("write2: {} {:?} {}", ino, data, flags);
        let f = self.get_file(&ino)?;
        //        f.write();
        let now = time::now().to_timespec();
        let old: Vec<u8> = f.get_content().clone().to_vec();
        let new_data: Vec<u8> = data.to_vec().clone();
        /*
        //let tail = new_data.len() as i64;
        let head = offset as usize;
        log::error!("head={} len={}", head, new_data.len());
        let bytes: Vec<_> = old.splice(head..new_data.len(), new_data)
            .collect();
            */

        let bytes: Vec<u8> = old
            .clone()
            .into_iter()
            .take(offset as usize)
            .chain(new_data.clone().into_iter())
            .chain(old.clone().into_iter().skip(new_data.len()))
            .collect();
        /*
        let z: Vec<u8> = old.to_vec();
        let mut new = z.iter().take(offset as usize).cloned().collect::<Vec<_>>();

        new.extend(data.iter().cloned().collect::<Vec<_>>());

        new.extend(
            z.iter()
            .skip(offset as usize + data.len())
            .collect::<Vec<_>>(),
            );
        log::error!("write bytes old={:?}", old);
        log::error!("write bytes new={:?}", new);
        */
        f.write(&bytes[..bytes.len() - 1]);
        let d: &[u8] = f.get_content();
        let s = d.len();

        self.files.entry(ino).and_modify(|node| {
            node.file_data.size = s as u64;
            node.file_data.ctime = now;
            node.file_data.atime = now;
        });

        let size = size as u32;
        Some(size)
    }
}
