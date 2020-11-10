#[macro_use]
extern crate lazy_static;

use std::{env, thread};

use webbrowser;
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
        static ref STORE: storage::Store = storage::Store::new();
    }

    {
        let p = env::current_dir()?;
        thread::spawn(move || uc::build_graph(&p, &STORE));
    }

    let bind_addr = format!("127.0.0.1:{}", 8080);
    //let _ = webbrowser::open(&format!("http://{}", bind_addr))?;

    {
        api::server(&bind_addr, &STORE)?.await
    }
}
