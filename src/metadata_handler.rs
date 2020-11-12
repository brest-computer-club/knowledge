use crate::domain::Metadata;
use crate::storage;
use async_std::{sync::Receiver, task};

pub fn watch(rch: &Receiver<Metadata>, store: &storage::Store) {
    task::block_on(async {
        loop {
            match rch.recv().await {
                Ok(p) => {
                    store.insert(&p.clone());
                }
                Err(e) => {
                    println!("metadata watch err: {}", e);
                    continue;
                }
            };
        }
    });
}
