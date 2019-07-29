use std::{path, collections};
use std::ffi::{OsStr, OsString};
use crate::fsys::inode::{Inode, NodeData, FileNode, RegularDirNode, DirNode};
use std::time::{SystemTime};
use crate::sw_svc::build_sw_service;
use crate::weather_svc::build_weather_service;

const UID: u32 = 1000;
const GID: u32 = 1000;
pub struct FileStore {
    file_table: collections::HashMap<u64, Inode>,
    ino_ctr: u64,
}

impl FileStore {
    pub fn new() -> FileStore {
        let mut f = FileStore {
            file_table: collections::HashMap::new(),
            ino_ctr: 2,
        };


        let data = RegularDirNode::new(); 
        let node_data = NodeData::RegularDir(data);
        let name = OsStr::new("root");
        let node = Inode::new(fuse::FUSE_ROOT_ID, node_data, &name, UID, GID);

        f.file_table.insert(1, node);

        let svc = build_sw_service();
        let weather_svc = build_weather_service();
        let one = 1;
        let svc_node = NodeData::ServiceDir(svc);
        let weather_node = NodeData::ServiceDir(weather_svc);
        f.add_child(&one, svc_node, OsStr::new("sw_svc"));
        f.add_child(&one, weather_node, OsStr::new("weather_svc"));

        f
    }

    pub fn _add_node(&mut self, _parent: &u64, node: &Inode, path: OsString) {
        self.file_table.entry(node.id).and_modify(|parent| {
            match &mut parent.data {
                NodeData::RegularDir(dir) => {
                    dir.add(node.id, path);
                }
                _ => ()
            }
        });
    }

    pub fn rename(&mut self, parent: &u64, name: &OsStr, newparent: u64, newname: &OsStr) {
        log::error!("{:?} {:?} {:?} {:?}", parent, name, newparent, newname);
        let id = self.remove_child(parent, name);

        self.file_table.entry(id).and_modify(|node| {
            node.path = path::Path::new(newname).to_path_buf();
        });

        let path = newname.to_os_string();
        self.file_table.entry(newparent).and_modify(|par| {
            match &mut par.data {
                NodeData::RegularDir(dir) => {
                    dir.add(id, path);
                }
                _ => log::error!("oopsie")
            }
        });
    }

    pub fn write(&mut self, ino: u64, data: &[u8], flags: u32, offset: i64) -> u32 {
        let str = String::from_utf8_lossy(data).trim().to_string();

        let size = std::mem::size_of_val(&str.as_bytes());
        log::error!("size={}", size);
        log::error!("write2: {} {:?} {}", ino, data, flags);

        self.file_table.entry(ino).and_modify(|f| {
            match &mut f.data {
                NodeData::File(file) => {

                    let now = SystemTime::now();
                    let old = &file.content;
                    let z: Vec<u8> = old.to_vec();
                    let mut new = z.iter().take(offset as usize).cloned().collect::<Vec<_>>();

                    new.extend(data.iter().cloned().collect::<Vec<_>>());

                    new.extend(
                        z.iter()
                        .skip(offset as usize + data.len())
                        .collect::<Vec<_>>(),
                        );
                    //let z: &[u8] = new.into();
                    file.content = new;
                    let d: &[u8] = &file.content;
                    let s = d.len();
                    f.attr.size = s as u64;
                    f.attr.ctime = now;
                    f.attr.atime = now;

                }
                _ => log::error!("oops")
            }
        });

        let size = size as u32;
        size
    }

    pub fn remove_child(&mut self, parent: &u64, name: &OsStr) -> u64 {
        let ino = self.resolve_path(parent, name).unwrap();
        log::error!("about to unlink: {}", ino);
        let id = ino.clone();

        self.file_table.entry(*parent).and_modify(|f| {
            match &mut f.data {
                NodeData::RegularDir(dir) => dir.remove(&id, name),
                _ => log::error!("can't rm file {:?}", parent)
            }
        });

        id
    }

    pub fn unlink(&mut self, parent: &u64, name: &OsStr) {
        let i = self.remove_child(parent, name);
        self.remove(&i);
        log::error!("{:?}", self.file_table);
    }

    pub fn create_dir(&mut self, parent: u64, name: &OsStr, _mode: u32) -> Option<&Inode> {
        let dir = RegularDirNode::new();
        let node = NodeData::RegularDir(dir);

        let id = self.add_child(&parent, node, name);
        self.get(&id)

    }

    pub fn remove(&mut self, id: &u64) {
        let node = self.file_table.get(id).unwrap();
        match &node.data {
            NodeData::RegularDir(dir) => {
                let y = dir.children.clone();
                for child in y.iter() {
                    self.remove(child);
                }
                self.file_table.remove(id);
            }
            _ => log::error!("no children")
        }

        self.file_table.remove(id);
    }

    pub fn get(&self, id: &u64) -> Option<&Inode> {
        self.file_table.get(id)
    }

    pub fn lookup_path(&mut self, parent: &u64, name: &OsStr) -> Option<&Inode> {
        let id = self.resolve_path(parent, name)?;

        self.file_table.entry(id).and_modify(|file| {
            file.access();
        });

        self.get(&id)
    }

    pub fn add_child(&mut self, parent_id: &u64, data: NodeData, name: &OsStr) -> u64 {
        let id: u64 = (self.ino_ctr) as u64;
        self.ino_ctr += 1;
        // replace with uid and gid from req
        let mut node = Inode::new(id, data, name, 1000, 1000);
        node.id = id;
        node.attr.ino = id;
        match &self.file_table.get(parent_id).unwrap().data {
            NodeData::RegularDir(_) => {
                self.file_table.insert(id, node);
            }
            NodeData::ServiceDir(dir) => {
                let data = dir.service.fetch_data(name.to_str());
                let d: &[u8] = &data.join("\n").into_bytes();
                let s = d.len();
                node.attr.size = s as u64;
                match &mut node.data {
                    NodeData::File(f) => {
                        f.content = data.join("\n").into_bytes();
                    }
                    _ => {
                        log::error!("oops");
                    }
                }


                self.file_table.insert(id, node);
            }
            _ => log::error!("not a dir")
        }

        // consider extracting to method
        // see rename above
        let path = name.to_os_string();
        self.file_table.entry(*parent_id).and_modify(|parent| {
            match &mut parent.data {
                NodeData::RegularDir(dir) => {
                    dir.add(id, path);
                }
                NodeData::ServiceDir(dir) => {
                    dir.add(id, path);
                }
                _ => ()
            }
        });
        log::debug!("new entry: {:?}", self.file_table);

        id
    }

    pub fn clear_file(&mut self, ino: &u64) {
        self.file_table.entry(*ino).and_modify(|f| {
            match &mut f.data {
                NodeData::File(file) => {
                    f.attr.size = 0;
                    file.content = [].to_vec();
                }
                _ => log::error!("Not a File")
            }
        });
    }


    pub fn touch_file(&mut self, parent: &u64, name: &OsStr) -> u64 {
        let file = FileNode::new();
        let node = NodeData::File(file);
        self.add_child(parent, node, name)
    }

    fn resolve_path(&self, parent: &u64, name: &OsStr) -> Option<u64> {
        let parent = self.get(parent).unwrap();
        match &parent.data {
            NodeData::RegularDir(dir) => {
                Some(dir.name_map.get(name)?.clone())
            }
            _ => None
        }
    }
}

