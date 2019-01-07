use crate::drakey_fs::inode;
use std::ffi;
use super::FileTree;
use libc::{ENOENT};
use std::collections;
use crate::drakey_fs::inode::{node_data::NodeData, node_types};

impl FileTree {
    fn add_node(&mut self, ino: &u64, node: inode::Inode, path: &ffi::OsStr) -> Result<&inode::Inode, i32> {
        match self.get_mut_dir(ino) {
            Some(parent) => {
                match parent.lookup_child(path) {
                    Some(_) => { println!("wat"); Err(1)},
                    None => {
                        let id = self.ino_ctr;
                        self.ino_ctr += 1;

                        self.files.insert(id, node);
                        self.attach_child(ino, path, &id);
                        match self.lookup(&id) {
                            Some(node) => Ok(node),
                            _ => {
                                log::error!("oopsie");
                                Err(ENOENT)
                            }
                        }

                    }
                }
            },
            None => Err(ENOENT)
        }
    }

    fn attach_child(&mut self, ino: &u64, path: &ffi::OsStr, child_id: &u64) -> Option<()> {
        let parent = self.get_mut_dir(ino)?;
        parent.children.insert(*child_id);
        parent.name_map.insert(path.to_os_string(), *child_id);
        Some(())
    }

    pub fn read_dir(&self, ino: &u64) -> Option<&collections::BTreeSet<u64>> {

        log::error!("read dir {}", ino);
        let f = self.lookup(ino)?;
        match &f.data {
            NodeData::Dir(dir) => Some(&dir.children),
            _ => None
        }
    }

    pub fn add_dir(&mut self, parent: &u64, path: &ffi::OsStr) -> Result<&inode::Inode, i32> {
        let node = inode::Inode::new_dir(&self.ino_ctr, path);
        log::error!("new dir node: {:?}", node);
        self.add_node(parent, node, path)
    }

    pub fn add_file(&mut self, parent: &u64, path: &ffi::OsStr) -> Result<&inode::Inode, i32> {
        let node = inode::Inode::new_file(&self.ino_ctr, path);
        self.add_node(parent, node, path)
    }

    pub fn lookup(&self, ino: &u64) -> Option<&inode::Inode> {
        self.files.get(ino)
    }

    fn resolve_path(&mut self, parent: &u64, path: &ffi::OsStr) -> Option<&u64> {
        let node = self.files.get_mut(parent)?; 
        match &mut node.data {
            NodeData::Dir(dir) => Some(dir.name_map.get(path).unwrap()),
            _ => None
        }
    }

    fn get_dir(&self, ino: &u64) -> Option<&node_types::DirNode> {
        let node = self.lookup(ino)?;
        match &node.data {
            NodeData::Dir(dir) => Some(dir),
            _ => None
        }
    }

    fn get_mut_dir(&mut self, ino: &u64) -> Option<&mut node_types::DirNode> {
        let node = self.files.get_mut(ino)?;
        match &mut node.data {
            NodeData::Dir(dir) => Some(dir),
            _ => None
        }
    }

    pub fn lookup_path(&self, ino: &u64, path: &ffi::OsStr ) -> Option<&inode::Inode> {
        let parent = self.get_dir(ino)?;
        let child_id = parent.name_map.get(path)?;
        log::error!("chid id: {}", child_id);
        self.lookup(child_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_file() {
        let f = FileTree::new();
        match f.lookup(&1) {
            Some(root) => assert_eq!(root.id, 1),
            None => panic!("root does not exist")
        }
    }

    #[test]
    fn lookup_invalid() {
        let f = FileTree::new();
        match f.lookup(&2) {
            Some(_invalid) => panic!("invalid ino"),
            None => assert!(true)
        }
    }

    #[test]
    fn add_dir() {
        let mut f = FileTree::new();
        match f.add_dir(&1, ffi::OsStr::new("thingy")) {
            Ok(dir) => assert_eq!(dir.id, 2),
            _ => panic!("mkdir failed")
        }
    }

    #[test]
    fn add_invalid_dir() {
        let mut f = FileTree::new();
        f.add_dir(&0, ffi::OsStr::new("thingy"));
        match f.add_dir(&0, ffi::OsStr::new("thingy")) {
            Ok(_id) => panic!("duplicate dir created"),
            _ => assert!(true)
        }
    }
}
