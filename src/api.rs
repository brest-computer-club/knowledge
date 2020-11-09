use actix_web::{web, HttpResponse, Responder};
use rust_embed::RustEmbed;

pub fn front_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(|| serve_static(StaticFile::Index, ()))));
    cfg.service(
        web::resource("/assets/elm.js").route(web::get().to(|| serve_static(StaticFile::Elm, ()))),
    );
}

pub fn back_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/tag/{tag}").route(web::get().to(|| get_by_tag())));
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
async fn get_by_tag() -> impl Responder {
    HttpResponse::Ok()
}
