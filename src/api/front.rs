use actix_web::{web, HttpResponse, Responder};
use rust_embed::RustEmbed;

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

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(|| serve_static(StaticFile::Index, ()))));
    cfg.service(
        web::resource("/elm.js").route(web::get().to(|| serve_static(StaticFile::Elm, ()))),
    );
}
