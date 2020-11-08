use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub fn watch(rch: &Receiver<PathBuf>, metach: &Sender<String>) {
    loop {
        match rch.recv() {
            Ok(p) => {
                let mc = Sender::clone(metach);
                thread::spawn(move || Handler.handle(&p.clone(), &mc));
            }
            Err(e) => {
                println!("file_dispatch err: {}", e);
                continue;
            }
        };
    }
}

struct Handler;

impl Handler {
    fn handle(&self, e: &PathBuf, metach: &Sender<String>) {
        let meta = match fs::metadata(e) {
            Ok(m) => m,
            Err(_) => return,
        };

        let _ = metach.send(format!("{:?}", meta.permissions()));
    }
}
