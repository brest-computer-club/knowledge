use std::env;
use std::io;

use crate::uc;
use std::fs;
use std::path::{Path, PathBuf};

pub trait Walker {
    fn get_root(&self) -> io::Result<PathBuf>;
    fn walk_tree(&self, dir: &Path, v: &dyn uc::Visitor);
}

pub struct W;

impl Walker for W {
    fn get_root(&self) -> io::Result<PathBuf> {
        env::current_dir()
    }

    fn walk_tree(&self, dir: &Path, v: &dyn uc::Visitor) {
        if dir.is_dir() {
            let ee = match fs::read_dir(dir) {
                Err(_) => return,
                Ok(ee) => ee,
            };

            for entry in ee {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue, // we just skip the entry in case of problem
                };

                let path = entry.path();
                if path.is_dir() {
                    self.walk_tree(&path, v);
                } else {
                    v.visit(&entry);
                }
            }
        }
    }
}
