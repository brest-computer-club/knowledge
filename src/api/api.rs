use actix_web::{App, HttpServer};
use webbrowser;

use crate::api::front;

#[actix_web::main]
async fn start_server(port: u32) -> std::io::Result<()> {
    let _ = webbrowser::open("http://localhost:8080");

    let address = format!("127.0.0.1:{}", port);
    println!("listening on http://localhost:{}", port);

    HttpServer::new(|| App::new().configure(front::routes))
        .bind(address)?
        .run()
        .await
}
