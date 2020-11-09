use std::sync::mpsc::Receiver;

use crate::domain::Metadata;
use crate::storage;

pub fn watch(rch: &Receiver<Metadata>, store: &storage::Store) {
    loop {
        match rch.recv() {
            Ok(p) => {
                store.insert(&p.clone());
            }
            Err(e) => {
                println!("metadata watch err: {}", e);
                continue;
            }
        };
    }
}
