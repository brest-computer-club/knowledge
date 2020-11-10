use crate::storage;
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use rust_embed::RustEmbed;

pub fn server(address: &str, store: &'static storage::Store) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .data(store.clone())
            .configure(back_routes)
            .configure(front_routes)
    })
    .bind(address)?
    .run();

    println!("listening on : {}", address);
    Ok(server)
}

pub fn front_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(|| serve_static(StaticFile::Index, ()))));
    cfg.service(
        web::resource("/assets/elm.js").route(web::get().to(|| serve_static(StaticFile::Elm, ()))),
    );
}

pub fn back_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/tag/{tag}").route(web::get().to(get_by_tag)));
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

// back routes

async fn get_by_tag(store: web::Data<storage::Store>, tag: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().json(store.get_by_tag(&tag.into_inner()))
}
