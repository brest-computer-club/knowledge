use std::env;
use std::io;

use crate::walker::Walker;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

pub struct W;

impl Walker for W {
    // for now, we use the working directory as the root folder
    // and throw an error if something goes wrong (permissions)
    fn get_root(&self) -> io::Result<PathBuf> {
        env::current_dir()
    }

    fn walk_tree(&self, dir: &Path, cb: &dyn Fn(&DirEntry)) {
        if dir.is_dir() {
            let ee = match fs::read_dir(dir) {
                Err(_) => return,
                Ok(ee) => ee,
            };

            for entry in ee {
                let entry = match entry {
                    // we just skip the entry in case of problem
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let path = entry.path();
                if path.is_dir() {
                    W.walk_tree(&path.to_path_buf(), cb);
                } else {
                    cb(&entry);
                }
            }
        }
    }
}
