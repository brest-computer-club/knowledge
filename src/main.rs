#[macro_use]
extern crate lazy_static;

use std::{env, thread};

mod api;
mod domain;
mod file_handler;
mod metadata_handler;
mod storage;
mod tree_traverser;
mod uc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    lazy_static! {
        pub static ref STORE: storage::Store = storage::Store::new();
    }

    {
        let p = env::current_dir()?;
        thread::spawn(move || uc::build_graph(&p, &STORE));
    }

    {
        api::server(&format!("127.0.0.1:{}", 8080), &STORE)?.await
    }
}
