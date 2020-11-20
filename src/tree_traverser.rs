use crate::domain::{FileEvent, FileOp};
use async_std::{
    sync::{Receiver, Sender},
    task,
};
use std::fs;
use std::path::PathBuf;

pub fn watch(dir_rcv: &Receiver<PathBuf>, dir_send: &Sender<PathBuf>, fe_send: &Sender<FileEvent>) {
    task::block_on(async {
        loop {
            if let Ok(p) = dir_rcv.recv().await {
                let dc = Sender::clone(dir_send);
                let fc = Sender::clone(fe_send);
                task::spawn(async move { traverse_tree(&p.clone(), &dc, &fc).await });
            };
        }
    });
}

async fn traverse_tree(dir: &PathBuf, dir_send: &Sender<PathBuf>, fe_send: &Sender<FileEvent>) {
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
                dir_send.send(path).await;
            } else {
                fe_send
                    .send(FileEvent {
                        op: FileOp::Create,
                        path,
                        dst: None,
                    })
                    .await;
            }
        }
    }
}
