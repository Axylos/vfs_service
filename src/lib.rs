use std::collections;
#[derive(Debug)]
struct NodeData {
    val: u64
}

#[derive(Debug)]
struct Inode {
    id: u64,
    data: NodeData,
    children: collections::HashSet<u64>
}

impl Inode {
    fn new(id: u64, data: NodeData) -> Inode {
        Inode {
            id,
            data,
            children: collections::HashSet::new()
        }
    }

    fn add(&mut self, id: u64) {
        self.children.insert(id);
    }

    fn inc(&mut self) {
        self.data.val += 1;
    }
}

struct FileMap {
    data: collections::HashMap<u64, Inode>
}

impl FileMap {
    fn add_child(&mut self, parent_id: &u64, data: NodeData) -> u64{
        let child = self.add(data);
        self.data.entry(*parent_id).and_modify(|parent| {
            parent.add(child);
        });

        child
    }
    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    fn inc(&mut self, id: &u64) {
        self.data.entry(*id).and_modify(|node| {
            node.inc();
        });
    }

    fn new() -> FileMap {
        FileMap {
            data: collections::HashMap::new()
        }
    }

    fn add(&mut self, data: NodeData) -> u64 {
        let id: u64 = (self.data.len() + 1) as u64;
        let node = Inode::new(id, data);
        self.data.insert(id, node);

        id
    }

    fn get(&self, id: &u64) -> Option<&Inode> {
        self.data.get(id)
    }

    fn remove(&mut self, id: &u64) {
        let x = &self.data.get(id).unwrap();

        let y = &x.children.clone();
        for child in y.iter() {
            self.remove(child);
        }
        self.data.remove(id);
    }

    fn has(&mut self, id: &u64) -> bool{
        self.data.contains_key(id)
    }
}

impl PartialEq for Inode {
    fn eq(&self, other: &Inode) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
#[test]
fn create_map() {
    let h = FileMap::new();
    assert!(h.is_empty());
}

#[test]
fn add_inode() {
    let mut h = FileMap::new();
    let node = NodeData { val: 1 };
    h.add(node);
    assert!(!h.is_empty());
}

#[test]
fn get_node() {
    let mut h = FileMap::new();
    let val = NodeData { val: 10 };
    let other_val = NodeData { val: 11 };

    h.add(val);
    h.add(NodeData { val: 11 });
    let node = h.get(&2).unwrap();
    assert_eq!(&node.data.val, &other_val.val);
}

#[test]
fn remove() {
    let mut h = FileMap::new();
    let val = NodeData { val: 10 };
    let id = h.add(val);
    h.remove(&id);
    assert!(h.is_empty());
}

fn build_with_children() -> FileMap {
    let mut h = FileMap::new();
    let val = NodeData { val: 10 };

    let id = h.add( val);

    let child = NodeData { val: 11 };
    h.add_child(&id, child);

    h
}

#[test]
fn add_child() {
    let mut h = FileMap::new();
    let val = NodeData { val: 10 };

    h.add(val);

    let node = NodeData { val: 12 };
    let child = h.add_child(&1, node);
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
    let child = NodeData { val: 12 };
    let another = NodeData { val: 13 };
    h.add_child(&1, child);
    h.add_child(&1, another);

    assert_eq!(h.data.len(), 4);

    h.remove(&1);
    println!("{:?}", h.data);
    assert!(h.is_empty());
}

#[test]
fn remove_nested_safely() {
    let mut h = build_with_children();
    let child = NodeData { val: 11 };
    let another = NodeData { val: 13 };
    let root = NodeData { val: 14 };
    let root_child = NodeData { val: 15 };
    h.add_child(&2, child);
    h.add_child(&2, another);
    let id = h.add(root);
    let root_child_id = h.add_child(&id, root_child);

    assert_eq!(h.data.len(), 6);

    h.remove(&1);
    assert_eq!(h.data.len(), 2);
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
