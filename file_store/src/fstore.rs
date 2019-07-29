use std::{path, collections};
use std::ffi::{OsStr, OsString};
use time::Timespec;
use time;
use crate::inode::{Inode};

extern crate file_node;

use log::*;

use file_node::{NodeData, gen_file_node, gen_dir_node, DirNode, ServiceDirNode, SingleService};

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

        let node_data = gen_dir_node();
        let name = OsStr::new("root");
        let node = Inode::new(fuse::FUSE_ROOT_ID, node_data, &name, UID, GID);

        f.file_table.insert(1, node);

        f
    }

    pub fn register_services(&mut self, svcs: Vec<Box<dyn SingleService + Send>>) {
        for svc in svcs {
            let n = svc.get_name();
            let name = OsStr::new(&n).clone();
            let node = ServiceDirNode::new(svc);
            let svc_node = NodeData::ServiceDir(node);
            let one = 1;

            self.add_child(&one, svc_node, OsStr::new(name));
        }

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

    pub fn rename(&mut self, parent: &u64, name: &OsStr, newparent: u64, newname: &OsStr) -> Result<u64, u64> {
        log::error!("{:?} {:?} {:?} {:?}", parent, name, newparent, newname);

        let mut result_id = 0;
        match self.remove_child(parent, name) {
            Some(id) => {
                self.file_table.entry(id).and_modify(|node| {
                    node.path = path::Path::new(newname).to_path_buf();
                });

                let path = newname.to_os_string();
                self.file_table.entry(newparent).and_modify(|par| {
                    match &mut par.data {
                        NodeData::RegularDir(dir) => {
                            dir.add(id, path);
                            result_id = id;
                        }
                        NodeData::ServiceDir(_) => {
                            log::error!("can't rename svc directory: {:?} {:?}", parent, name);
                        }
                        NodeData::File(file) => {
                            log::error!("new parent a file and not a valid directory: {:?}", file)
                        }
                    }
                });
            }

            None => log::error!("can't delete file: {:?}", name)
        }
        if result_id > 0 {
            Ok(result_id)
        } else {
            Err(0)
        }
    }

    pub fn write(&mut self, ino: u64, data: &[u8], flags: u32, offset: i64) -> u32 {
        let str = String::from_utf8_lossy(data).trim().to_string();

        let size = std::mem::size_of_val(&str.as_bytes());
        log::error!("size={}", size);
        log::error!("write2: {} {:?} {}", ino, data, flags);

        self.file_table.entry(ino).and_modify(|f| {
            match &mut f.data {
                NodeData::File(file) => {

                    let now = time::get_time();
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

    pub fn remove_child(&mut self, parent: &u64, name: &OsStr) -> Option<u64> {
        let ino = self.resolve_path(parent, name).unwrap();
        log::error!("about to unlink: {}", ino);
        let id = ino.clone();

        let mut ok = false;
        self.file_table.entry(*parent).and_modify(|f| {
            match &mut f.data {
                NodeData::RegularDir(dir) => {
                    dir.remove(&id, name);
                    ok = true;
                }
                _ => {
                    log::error!("can't rm file {:?}", parent);
                    ok = false;
                }
            }
        });

        if ok {
            Some(id)
        } else {
            None
        }
    }

    pub fn unlink(&mut self, parent: &u64, name: &OsStr) {
        match self.remove_child(parent, name) {
            Some(i) => self.remove(&i),
            None => log::error!("{:?}", self.file_table)
        }
    }

    pub fn create_dir(&mut self, parent: u64, name: &OsStr, _mode: u32) -> Option<&Inode> {
        let node = gen_dir_node();

        let id = self.add_child(&parent, node, name);
        self.get(&id)

    }

    pub fn read_dir_children(&self, ino: &u64) -> Option<&collections::BTreeSet<u64>> {
        match self.get(ino) {
            Some(inode) => {
                match &inode.data {
                    NodeData::RegularDir(node) => {
                        Some(&node.children)
                    }
                    NodeData::ServiceDir(node) => {
                        Some(&node.children)
                    }
                    NodeData::File(_) => {
                        log::error!("file found during read dir lookup: {:?}", ino);
                        None
                    }
                }
            }
            None => None
        }

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

    // this is troubling; see the call at self.store.read_file in fuse_system
    // there a buf is initialized as not mutable, but can be "safely" passed as
    // as an argument construed as a mut Vec<u8>
    pub fn read_file(&self, ino: &u64) -> Option<Vec<u8>> {
        match self.get(ino) {
            Some(f) => {
                match &f.data {
                    NodeData::File(file) => {
                        let data = &file.content;
                        Some(data.to_vec())
                    }
                _ => None
                }

            }
            None => {
                log::error!("read failed {:?}", ino);
                None
            }
        }
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
        log::info!("new entry: {:?}", self.file_table);

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
        let node = gen_file_node();
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

