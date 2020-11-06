use actix_web::{App, HttpServer};
use std::thread;
use webbrowser;

mod api;
mod walker;
use walker::local;

mod uc;
use uc::graph;

#[actix_web::main]
async fn start_server(port: u32) -> std::io::Result<()> {
    let address = format!("127.0.0.1:{}", port);
    println!("listening on http://localhost:{}", port);

    HttpServer::new(|| App::new().configure(api::front::routes))
        .bind(address)?
        .run()
        .await
}

fn main() -> std::io::Result<()> {
    thread::spawn(|| {
        let w = &local::W;
        let _ = graph::build(w);
    });

    let _ = webbrowser::open("http://localhost:8080");
    start_server(8080)
}
