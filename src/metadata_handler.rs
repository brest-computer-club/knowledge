use crate::domain::MetadataEvent;
use crate::storage;
use async_std::{sync::Receiver, task};

pub fn watch(rch: &Receiver<MetadataEvent>, store: &storage::Store) {
    task::block_on(async {
        loop {
            match rch.recv().await {
                Ok(me) => match me {
                    MetadataEvent::Create(m) => store.insert(&m),
                    MetadataEvent::Remove(p) => store.remove(&p),
                    MetadataEvent::Move(src, dst) => store.update_path(&src, &dst),
                    MetadataEvent::Changed(m) => store.update_meta(&m),
                },
                Err(_) => {
                    continue;
                }
            };
        }
    });
}
