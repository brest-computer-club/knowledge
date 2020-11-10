use crate::storage;
use crate::uc;
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use rust_embed::RustEmbed;
extern crate base64;

pub fn server(address: &str, store: &'static storage::Store) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .data(store.clone())
            .configure(back_routes)
            .configure(static_routes)
    })
    .bind(address)?
    .run();

    println!("listening on : {}", address);
    Ok(server)
}

fn static_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(|| serve_static(StaticFile::Index, ()))));
    cfg.service(
        web::resource("/assets/elm.js").route(web::get().to(|| serve_static(StaticFile::Elm, ()))),
    );
}

fn back_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/tags").route(web::get().to(get_tags)));
    cfg.service(web::resource("/tag/{tag}").route(web::get().to(get_by_tag)));
    cfg.service(web::resource("/article/{path}").route(web::get().to(get_article_by_path)));
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

async fn get_tags(store: web::Data<storage::Store>) -> impl Responder {
    let resp = uc::get_all_tags(&store).unwrap();
    HttpResponse::Ok().json(resp)
}

async fn get_article_by_path(path: web::Path<String>) -> impl Responder {
    let dec = base64::decode(&path.into_inner()).unwrap();
    let p = String::from_utf8(dec).unwrap();
    let resp = uc::get_article_content(&p).unwrap();
    HttpResponse::Ok().body(resp)
}
