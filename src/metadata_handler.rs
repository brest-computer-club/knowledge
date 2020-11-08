use std::sync::mpsc::Receiver;
use std::thread;

pub fn watch(rch: &Receiver<String>) {
    loop {
        match rch.recv() {
            Ok(p) => {
                thread::spawn(move || MetaStorer.handle(&p.clone()));
            }
            Err(e) => {
                println!("meta_dispatch err: {}", e);
                continue;
            }
        };
    }
}

struct MetaStorer;

impl MetaStorer {
    fn handle(&self, e: &String) {
        println!("meta {:?}", e);
    }
}
