use async_std::{
    sync::{Receiver, Sender},
    task,
};
use std::fs;
use std::path::PathBuf;

pub fn watch(rch: &Receiver<PathBuf>, fchan: &Sender<PathBuf>, dchan: &Sender<PathBuf>) {
    task::block_on(async {
        loop {
            match rch.recv().await {
                Ok(p) => {
                    let fc = Sender::clone(fchan);
                    let dc = Sender::clone(dchan);
                    task::spawn(async move { traverse_tree(&p.clone(), &fc, &dc).await });
                }
                Err(e) => {
                    println!("tree_traverser watch err: {}", e);
                    continue;
                }
            };
        }
    });
}

async fn traverse_tree(dir: &PathBuf, fchan: &Sender<PathBuf>, dchan: &Sender<PathBuf>) {
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
                dchan.send(path).await;
            } else {
                fchan.send(path).await;
            }
        }
    }
}
