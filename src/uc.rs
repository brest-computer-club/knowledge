use crate::walker;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct Interactor<'a> {
    pub visitor: &'a dyn Visitor,
}

impl<'a> Interactor<'a> {
    pub fn build_graph(&self, r: &Path, w: &impl walker::Walker) {
        w.walk_tree(&r, self.visitor);
    }
}

pub trait Visitor {
    fn visit(&self, e: &DirEntry);
}

pub struct FileVisitor {
    pub files: Arc<Mutex<Vec<PathBuf>>>,
}

impl Visitor for FileVisitor {
    fn visit(&self, e: &DirEntry) {
        let mut l = self.files.lock();
        match l {
            Ok(ref mut ff) => {
                ff.push(e.path());
            }
            Err(e) => {
                println!("problem occured with entry {:?}", e);
            }
        }
    }
}

#[cfg(test)]
impl FileVisitor {
    fn get_stored(&self) -> Vec<PathBuf> {
        self.files.lock().unwrap().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn uc_basic() -> std::io::Result<()> {
        let dir = tempdir()?;
        File::create(dir.path().join("file1"))?;
        File::create(dir.path().join("file2"))?;

        let v = &FileVisitor {
            files: Arc::new(Mutex::new(vec![])),
        };

        let i = Interactor { visitor: v };

        i.build_graph(dir.path(), &walker::W);
        assert_eq!(v.get_stored().len(), 2);

        Ok(())
    }
}
