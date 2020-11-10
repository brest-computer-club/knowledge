#[macro_use]
extern crate lazy_static;

use actix_web::{App, HttpServer};
use std::{env, thread};

mod api;
mod conf;
mod domain;
mod file_handler;
mod metadata_handler;
mod storage;
mod tree_traverser;
mod uc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    {
        let p = env::current_dir()?;
        thread::spawn(move || uc::build_graph(&p, &conf::STORE));
    }

    {
        let address = format!("127.0.0.1:{}", 8080);
        println!("listening on : {}", address);

        HttpServer::new(|| {
            App::new()
                .configure(api::front_routes)
                .configure(api::back_routes)
        })
        .bind(address)?
        .run()
        .await
    }
}
