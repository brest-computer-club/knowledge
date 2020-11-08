use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub fn watch(rch: &Receiver<PathBuf>, fchan: &Sender<PathBuf>, dchan: &Sender<PathBuf>) {
    loop {
        match rch.recv() {
            Ok(p) => {
                let fc = Sender::clone(fchan);
                let dc = Sender::clone(dchan);
                thread::spawn(move || traverse_tree(&p.clone(), &fc, &dc));
            }
            Err(e) => {
                println!("tree_traverser watch err: {}", e);
                continue;
            }
        };
    }
}

fn traverse_tree(dir: &PathBuf, fchan: &Sender<PathBuf>, dchan: &Sender<PathBuf>) {
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
                let _ = dchan.send(path);
            } else {
                let _ = fchan.send(path);
            }
        }
    }
}
