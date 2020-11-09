use std::sync::mpsc::Receiver;
use std::thread;

use crate::domain::Metadata;

pub fn watch(rch: &Receiver<Metadata>) {
    loop {
        match rch.recv() {
            Ok(p) => {
                thread::spawn(move || store_metadata(&p.clone()));
            }
            Err(e) => {
                println!("metadata watch err: {}", e);
                continue;
            }
        };
    }
}

fn store_metadata(e: &Metadata) {
    println!("meta {:?}", e);
}
