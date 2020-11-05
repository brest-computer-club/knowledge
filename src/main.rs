use actix_web::{App, HttpServer};

mod api;

#[actix_web::main]
async fn start_server(port: i32) -> std::io::Result<()> {
    let address = format!("127.0.0.1:{}", port);
    println!("listening on localhost:{}", port);

    HttpServer::new(|| App::new().configure(api::front::routes))
        .bind(address)?
        .run()
        .await
}

fn main() -> std::io::Result<()> {
    start_server(8080)
}
