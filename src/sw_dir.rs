use crate::fsys::inode::{ServiceDirNode};

pub struct SwDir {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}
