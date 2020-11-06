use actix_web::{App, HttpServer};

use crate::api::front;

#[actix_web::main]
pub async fn start_server(port: u32) -> std::io::Result<()> {
    let address = format!("127.0.0.1:{}", port);
    println!("listening on http://localhost:{}", port);

    HttpServer::new(|| App::new().configure(front::routes))
        .bind(address)?
        .run()
        .await
}
