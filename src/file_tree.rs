use fuse::{FileAttr, FileType};
use std::collections;
use std::collections::hash_map::Entry;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path;
use time::Timespec;

#[derive(Debug)]
pub struct NodeData {
    pub file_data: FileAttr,
    pub content: Vec<u8>,
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
        self.data.file_data.ino += 1;
    }
}

pub struct FileMap {
    data: collections::HashMap<u64, Inode>,
}

impl FileMap {
    pub fn clear_file(&mut self, ino: &u64) {
        self.data.entry(*ino).and_modify(|f| {
            f.data.file_data.size = 0;
            f.data.content = [].to_vec();
        });
    }

    pub fn write(&mut self, ino: u64, data: &[u8], flags: u32, offset: i64) -> u32 {
        let str = String::from_utf8_lossy(data).trim().to_string();

        let size = std::mem::size_of_val(&str.as_bytes());
        log::error!("size={}", size);
        log::error!("write2: {} {:?} {}", ino, data, flags);

        self.data.entry(ino).and_modify(|f| {
            let now = time::now().to_timespec();
            let old = &f.data.content;
            let z: Vec<u8> = old.to_vec();
            let mut new = z.iter().take(offset as usize).cloned().collect::<Vec<_>>();

            new.extend(data.iter().cloned().collect::<Vec<_>>());

            new.extend(
                z.iter()
                    .skip(offset as usize + data.len())
                    .collect::<Vec<_>>(),
            );
            //let z: &[u8] = new.into();
            f.data.content = new;
            let d: &[u8] = &f.data.content;
            let s = d.len();
            f.data.file_data.size = s as u64;
            f.data.file_data.ctime = now;
            f.data.file_data.atime = now;
        });

        let size = size as u32;
        size
    }
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

    fn resolve_path(&self, parent: &u64, name: &OsStr) -> Option<&u64> {
        let parent = self.get(parent).unwrap();
        parent.name_map.get(name)
    }

    pub fn lookup_path(&mut self, parent: &u64, name: &OsStr) -> Option<&Inode> {
        // double the work so the result of an
        // immutable borrow is not used for a mutable borrow
        let id = self.resolve_path(parent, name)?;

        self.data.entry(*id).and_modify(|file| {
            file.access();
        });

        let id = self.resolve_path(parent, name).unwrap();
        self.get(&id)
    }

    pub fn touch_file(&mut self, parent: &u64, name: &OsStr) -> u64 {
        let mut file = build_dummy_file();
        file.kind = FileType::RegularFile;
        let node = NodeData {
            content: Vec::new(),
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
            content: Vec::new(),
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

    pub fn get_mut(&mut self, id: &u64) -> Option<&mut Inode> {
        self.data.get_mut(id)
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
    let ino = 1;
    FileAttr {
        ino,
        kind: FileType::Directory,
        nlink: 0,
        perm: 0o755,
        rdev: 0,
        size: 0,
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
    let node = build_dummy_node();
    h.add(node);
    assert!(!h.is_empty());
}

#[test]
fn get_node() {
    let mut h = FileMap::new();
    let val = build_dummy_node();
    let _other_val = build_dummy_node();

    h.add(val);
    h.add(build_dummy_node());

    let node = h.get(&3).unwrap();
    assert_eq!(&node.data.file_data.ino, &3);
}

#[test]
fn remove() {
    let mut h = FileMap::new();
    let val = build_dummy_node();
    let id = h.add(val);
    h.remove(&id);
    assert_eq!(h.data.len(), 1);
}

#[cfg(test)]
fn build_with_children() -> FileMap {
    let mut h = FileMap::new();
    let val = build_dummy_node();

    let id = h.add(val);

    let child = build_dummy_node();
    h.add_child(&id, child, &OsStr::new("fake file"));

    h
}

#[test]
fn add_child() {
    let mut h = FileMap::new();
    let val = build_dummy_node();

    h.add(val);

    let node = build_dummy_node();
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
    let child = build_dummy_node();
    let another = build_dummy_node();
    h.add_child(&1, child, &OsString::from("child"));
    h.add_child(&1, another, &OsString::from("another"));

    assert_eq!(h.data.len(), 5);

    h.remove(&1);
    assert!(h.is_empty());
}

fn build_dummy_node() -> NodeData {
    NodeData {
        content: Vec::new(),
        file_data: build_dummy_file(),
    }
}
#[test]
fn remove_nested_safely() {
    let mut h = build_with_children();
    let child = build_dummy_node();
    let another = build_dummy_node();
    let root = build_dummy_node();
    let root_child = build_dummy_node();
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
    let old = h.get(&1).unwrap().data.file_data.ino;
    h.inc(&1);
    let new = h.get(&1).unwrap().data.file_data.ino;
    assert_eq!(old, new - 1);
}
