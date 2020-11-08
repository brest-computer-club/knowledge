use crate::file_handler;
use crate::metadata_handler;
use crate::tree_traverser;

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub fn build_graph(p: &PathBuf) {
    let (dir_send, dir_recv): (Sender<PathBuf>, Receiver<PathBuf>) = channel();
    let (file_send, file_recv): (Sender<PathBuf>, Receiver<PathBuf>) = channel();
    let (meta_send, meta_recv): (Sender<String>, Receiver<String>) = channel();

    let ds = dir_send.clone();
    thread::spawn(move || tree_traverser::watch(&dir_recv, &file_send, &ds));
    thread::spawn(move || file_handler::watch(&file_recv, &meta_send));
    thread::spawn(move || metadata_handler::watch(&meta_recv));

    let _ = dir_send.send(p.clone());
}
