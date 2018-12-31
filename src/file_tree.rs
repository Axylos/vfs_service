use fuse::{FileAttr, FileType};
use std::collections;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path;
use time::Timespec;

#[derive(Debug)]
pub struct NodeData {
    pub file_data: FileAttr,
}

#[derive(Debug)]
pub struct Inode {
    pub id: u64,
    pub ttl: Timespec,
    pub data: NodeData,
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
    pub path: path::PathBuf,
}

impl Inode {
    pub fn access(&mut self) {
        let now = time::now().to_timespec();
        self.data.file_data.atime = now;
        log::error!("ino={} now={:?} atime={:?}", self.id, now, self.data.file_data.atime);
    }
    fn new(id: u64, data: NodeData, name: &OsStr) -> Inode {
        let ttl = time::now().to_timespec() + time::Duration::hours(10);
        let path = path::PathBuf::from(name);
        Inode {
            id,
            path,
            data,
            ttl,
            children: collections::BTreeSet::new(),
            name_map: collections::HashMap::new(),
        }
    }

    fn add(&mut self, id: u64, name: std::ffi::OsString) {
        self.children.insert(id);
        self.name_map.insert(name, id);
    }

    #[cfg(test)]
    fn inc(&mut self) {
        self.data.val += 1;
    }
}

pub struct FileMap {
    data: collections::HashMap<u64, Inode>,
}

impl FileMap {
    pub fn access_file(&mut self, ino: &u64) {
        self.data.entry(*ino).and_modify(|file| {
            file.access();
        });
    }

    pub fn add_child(&mut self, parent_id: &u64, data: NodeData, name: &OsStr) -> u64 {
        let id: u64 = (self.data.len() + 1) as u64;
        let mut node = Inode::new(id, data, name);
        node.id = id;
        node.data.file_data.ino = id;
        self.data.insert(id, node);
        let path = name.to_os_string();
        self.data.entry(*parent_id).and_modify(|parent| {
            parent.add(id, path);
        });
        log::error!("new entry: {:?}", self.data);

        id
    }

    pub fn lookup_path(&mut self, parent: &u64, name: &OsStr) -> Option<&Inode> {
        let parent = self.get(parent).unwrap();
        log::error!("lookup args: {:?} {:?}", parent.name_map, name);
        let id = parent.name_map.get(name)?;
        self.get(id)
    }

    pub fn touch_file(&mut self, parent: &u64, name: &OsStr) -> u64 {
        let mut file = build_dummy_file();
        file.kind = FileType::RegularFile;
        let node = NodeData {
            file_data: file,
        };
        self.add_child(parent, node, name)
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    #[cfg(test)]
    fn inc(&mut self, id: &u64) {
        self.data.entry(*id).and_modify(|node| {
            node.inc();
        });
    }

    pub fn new() -> FileMap {
        let mut f = FileMap {
            data: collections::HashMap::new(),
        };

        let data = NodeData {
            file_data: build_dummy_file(),
        };
        let name = OsStr::new("root");
        let node = Inode::new(1, data, &name);
        f.data.insert(1, node);

        f
    }

    #[cfg(test)]
    pub fn add(&mut self, data: NodeData) -> u64 {
        self.add_child(&1, data, OsStr::new("root"))
    }

    pub fn get(&self, id: &u64) -> Option<&Inode> {
        self.data.get(id)
    }

    #[cfg(test)]
    pub fn remove(&mut self, id: &u64) {
        let x = &self.data.get(id).unwrap();

        let y = &x.children.clone();
        for child in y.iter() {
            self.remove(child);
        }
        self.data.remove(id);
    }

    #[cfg(test)]
    pub fn has(&mut self, id: &u64) -> bool {
        self.data.contains_key(id)
    }
}

impl PartialEq for Inode {
    fn eq(&self, other: &Inode) -> bool {
        self.id == other.id
    }
}

fn build_dummy_file() -> FileAttr {
    let ts = time::now().to_timespec();
    let _ttl = Timespec::new(1, 0);
    let ino = 1;
    FileAttr {
        ino,
        kind: FileType::Directory,
        nlink: 0,
        perm: 0o755,
        rdev: 0,
        size: 100,
        atime: ts,
        ctime: ts,
        crtime: ts,
        mtime: ts,
        blocks: 100,
        flags: 0,
        gid: 0,
        uid: 0,
    }
}
#[cfg(test)]
#[test]
fn create_map() {
    let h = FileMap::new();
    assert_eq!(h.data.len(), 1);
}

#[test]
fn add_inode() {
    let mut h = FileMap::new();
    let node = NodeData {
        val: 1,
        file_data: build_dummy_file(),
    };
    h.add(node);
    assert!(!h.is_empty());
}

#[test]
fn get_node() {
    let mut h = FileMap::new();
    let val = NodeData {
        val: 10,
        file_data: build_dummy_file(),
    };
    let other_val = NodeData {
        val: 11,
        file_data: build_dummy_file(),
    };

    h.add(val);
    h.add(NodeData {
        val: 11,
        file_data: build_dummy_file(),
    });
    let node = h.get(&3).unwrap();
    assert_eq!(&node.data.val, &other_val.val);
}

#[test]
fn remove() {
    let mut h = FileMap::new();
    let val = NodeData {
        val: 10,
        file_data: build_dummy_file(),
    };
    let id = h.add(val);
    h.remove(&id);
    assert_eq!(h.data.len(), 1);
}

#[cfg(test)]
fn build_with_children() -> FileMap {
    let mut h = FileMap::new();
    let val = NodeData {
        val: 10,
        file_data: build_dummy_file(),
    };

    let id = h.add(val);

    let child = NodeData {
        val: 11,
        file_data: build_dummy_file(),
    };
    h.add_child(&id, child, &OsStr::new("fake file"));

    h
}

#[test]
fn add_child() {
    let mut h = FileMap::new();
    let val = NodeData {
        val: 10,
        file_data: build_dummy_file(),
    };

    h.add(val);

    let node = NodeData {
        val: 12,
        file_data: build_dummy_file(),
    };
    let child = h.add_child(&1, node, &OsString::from("node"));
    let parent = h.get(&1).unwrap();
    assert!(parent.children.contains(&child));
}

#[test]
fn remove_with_children() {
    let mut h = build_with_children();

    assert!(h.has(&2));
    h.remove(&1);

    assert!(!h.has(&2));
}

#[test]
fn remove_nested_children() {
    let mut h = build_with_children();
    let child = NodeData {
        val: 12,
        file_data: build_dummy_file(),
    };
    let another = NodeData {
        val: 13,
        file_data: build_dummy_file(),
    };
    h.add_child(&1, child, &OsString::from("child"));
    h.add_child(&1, another, &OsString::from("another"));

    assert_eq!(h.data.len(), 5);

    h.remove(&1);
    assert!(h.is_empty());
}

#[test]
fn remove_nested_safely() {
    let mut h = build_with_children();
    let child = NodeData {
        val: 11,
        file_data: build_dummy_file(),
    };
    let another = NodeData {
        val: 13,
        file_data: build_dummy_file(),
    };
    let root = NodeData {
        val: 14,
        file_data: build_dummy_file(),
    };
    let root_child = NodeData {
        val: 15,
        file_data: build_dummy_file(),
    };
    h.add_child(&2, child, &OsString::from("stuff"));
    h.add_child(&2, another, &OsString::from("jerry"));
    let id = h.add(root);
    let root_child_id = h.add_child(&id, root_child, &OsString::from("root"));

    assert_eq!(h.data.len(), 7);

    h.remove(&2);
    assert_eq!(h.data.len(), 3);
    assert!(h.has(&root_child_id));
}

#[test]
fn inc_data() {
    let mut h = build_with_children();
    let old = h.get(&1).unwrap().data.val;
    h.inc(&1);
    let new = h.get(&1).unwrap().data.val;
    assert_eq!(old, new - 1);
}
