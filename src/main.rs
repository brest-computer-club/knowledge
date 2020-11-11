use clap::App;
use lazy_static::lazy_static;

use rand::Rng;
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
    let matches = App::new("Knowledge")
        .author("Brest Computer Club <brest-computer-club@protonmail.com>")
        .about("transform your markdown file into a knowledge base")
        .arg("-p [--port] 'the port to use app'")
        .get_matches();

    lazy_static! {
        static ref STORE: storage::Store = storage::Store::new();
    }

    {
        let p = env::current_dir()?;
        thread::spawn(move || uc::build_graph(&p, &STORE));
    }

    {
        let mut rng = rand::thread_rng();
        let port: u16 = matches
            .value_of_t("p")
            .unwrap_or(rng.gen_range(3000, 10000));

        let bind_addr = format!("127.0.0.1:{}", port);
        let _ = webbrowser::open(&format!("http://{}", bind_addr));

        api::server(&bind_addr, &STORE)?.await
    }
}
