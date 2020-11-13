use async_std::sync::{channel, Receiver, Sender};
use async_std::task;
use std::fs;
use std::path::PathBuf;
use std::thread;

use crate::domain::Metadata;
use crate::file_handler;
use crate::metadata_handler;
use crate::storage;
use crate::tree_traverser;

pub fn build_graph(p: &PathBuf, store: &'static storage::Store) {
    let (dir_send, dir_recv): (Sender<PathBuf>, Receiver<PathBuf>) = channel(1000);
    let (file_send, file_recv): (Sender<PathBuf>, Receiver<PathBuf>) = channel(1000);
    let (meta_send, meta_recv): (Sender<Metadata>, Receiver<Metadata>) = channel(1000);

    let ds = dir_send.clone();
    thread::spawn(move || tree_traverser::watch(&dir_recv, &file_send, &ds));
    thread::spawn(move || file_handler::watch(&file_recv, &meta_send));
    thread::spawn(move || metadata_handler::watch(&meta_recv, store));

    task::block_on(async { dir_send.send(p.clone()).await });
}

// todo : path is not checked, do not expose this publicly
pub fn get_article_content(p: &str) -> std::io::Result<String> {
    let content = fs::read_to_string(p)?;
    Ok(content)
}
