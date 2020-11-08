use std::sync::mpsc::Receiver;
use std::thread;

pub fn watch(rch: &Receiver<String>) {
    loop {
        match rch.recv() {
            Ok(p) => {
                thread::spawn(move || store_metadata(&p.clone()));
            }
            Err(e) => {
                println!("metadata watch err: {}", e);
                continue;
            }
        };
    }
}

fn store_metadata(e: &String) {
    println!("meta {:?}", e);
}
