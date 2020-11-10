use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::domain::Metadata;
use crate::file_handler;
use crate::metadata_handler;
use crate::storage;
use crate::tree_traverser;

pub fn build_graph(p: &PathBuf, store: &'static storage::Store) {
    let (dir_send, dir_recv): (Sender<PathBuf>, Receiver<PathBuf>) = channel();
    let (file_send, file_recv): (Sender<PathBuf>, Receiver<PathBuf>) = channel();
    let (meta_send, meta_recv): (Sender<Metadata>, Receiver<Metadata>) = channel();

    let ds = dir_send.clone();
    thread::spawn(move || tree_traverser::watch(&dir_recv, &file_send, &ds));
    thread::spawn(move || file_handler::watch(&file_recv, &meta_send));
    thread::spawn(move || metadata_handler::watch(&meta_recv, store));

    let _ = dir_send.send(p.clone());
}

pub fn get_article_content(p: &str) -> std::io::Result<String> {
    let content = fs::read_to_string(p)?;
    Ok(content)
}

pub fn get_all_tags(store: &storage::Store) -> std::io::Result<Vec<String>> {
    Ok(store.get_all_tags())
}
