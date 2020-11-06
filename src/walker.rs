pub mod local;

use std::fs::DirEntry;
use std::io;
use std::path::{Path, PathBuf};

pub trait Walker {
    fn get_root(&self) -> io::Result<PathBuf>;
    fn walk_tree(&self, dir: &Path, cb: &dyn Fn(&DirEntry));
}
