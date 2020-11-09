use actix_web::{App, HttpServer};
use std::{env, thread, time};

mod api;
mod domain;
mod file_handler;
mod metadata_handler;
mod storage;
mod tree_traverser;
mod uc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let store = &storage::Store::new();

    {
        let p = env::current_dir()?;
        let s = store.clone();
        thread::spawn(move || uc::build_graph(&p, &s));
    }

    {
        let s = store.clone();
        thread::spawn(move || loop {
            thread::sleep(time::Duration::from_secs(1));
            println!("{:?}", s.get_by_tag(&"k8s".into()));
        });
    }

    let address = format!("127.0.0.1:{}", 8080);
    println!("{}", address);

    HttpServer::new(|| {
        App::new()
            .configure(api::front_routes)
            .configure(api::back_routes)
    })
    .bind(address)?
    .run()
    .await
}
