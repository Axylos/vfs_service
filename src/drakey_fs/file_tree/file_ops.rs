use super::FileTree;
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
}
