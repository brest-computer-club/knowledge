use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::domain::Metadata;
use crate::file_handler;
use crate::metadata_handler;
use crate::storage;
use crate::tree_traverser;

pub fn build_graph(p: &PathBuf, store: &storage::Store) {
    let (dir_send, dir_recv): (Sender<PathBuf>, Receiver<PathBuf>) = channel();
    let (file_send, file_recv): (Sender<PathBuf>, Receiver<PathBuf>) = channel();
    let (meta_send, meta_recv): (Sender<Metadata>, Receiver<Metadata>) = channel();

    let ds = dir_send.clone(); // cloned because it's used both by tree_traverser & this fn (below)
    let s = store.clone();
    thread::spawn(move || tree_traverser::watch(&dir_recv, &file_send, &ds));
    thread::spawn(move || file_handler::watch(&file_recv, &meta_send));
    thread::spawn(move || metadata_handler::watch(&meta_recv, &s));

    let _ = dir_send.send(p.clone());
}
