use actix_web::{web, HttpResponse, Responder};
use rust_embed::RustEmbed;

use actix_web::{App, HttpServer};

#[actix_web::main]
pub async fn start_server(port: u32) -> std::io::Result<()> {
    let address = format!("127.0.0.1:{}", port);
    println!("listening on http://localhost:{}", port);

    HttpServer::new(|| App::new().configure(front_routes))
        .bind(address)?
        .run()
        .await
}

// frontend routes
#[derive(RustEmbed)]
#[folder = "./front/public"]
struct Asset;

enum StaticFile {
    Index,
    Elm,
}

async fn serve_static(f: StaticFile, _: ()) -> impl Responder {
    let path = match f {
        StaticFile::Index => "index.html",
        StaticFile::Elm => "elm.js",
    };

    let file = Asset::get(path).unwrap();
    HttpResponse::Ok().body(file)
}

fn front_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(|| serve_static(StaticFile::Index, ()))));
    cfg.service(
        web::resource("/assets/elm.js").route(web::get().to(|| serve_static(StaticFile::Elm, ()))),
    );
}
