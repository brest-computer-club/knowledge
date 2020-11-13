use crate::storage;
use crate::uc;
use actix_cors::Cors;
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use base64;
use rust_embed::RustEmbed;

pub fn server(
    address: &str,
    store: &'static storage::Store,
    dev_mode: &bool,
) -> Result<Server, std::io::Error> {
    let dev_mode = dev_mode.clone();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(get_cors(&dev_mode))
            .data(store.clone())
            .configure(static_routes)
            .configure(back_routes)
    })
    .bind(address)?
    .run();

    println!("listening on : {}", address);
    Ok(server)
}

fn get_cors(dev_mode: &bool) -> Cors {
    if *dev_mode {
        return Cors::default()
            .allowed_origin("http://localhost:8000")
            .allowed_methods(vec!["GET"]);
    }
    Cors::default()
}

fn static_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(|| serve_static(StaticFile::Index, ()))));
    cfg.service(
        web::resource("/elm.js").route(web::get().to(|| serve_static(StaticFile::Elm, ()))),
    );
}

fn back_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/tags", web::get().to(get_all_tags))
            .service(web::resource("/tags/{tag}").route(web::get().to(get_by_tag)))
            .service(web::resource("/articles").route(web::get().to(get_all_articles)))
            .service(web::resource("/articles/{path}").route(web::get().to(get_article_by_path)))
            .service(web::resource("/images/{path}").route(web::get().to(get_asset_by_path))),
    );
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

async fn get_all_articles(store: web::Data<storage::Store>) -> impl Responder {
    HttpResponse::Ok().json(store.get_all_articles())
}

async fn get_all_tags(store: web::Data<storage::Store>) -> impl Responder {
    HttpResponse::Ok().json(store.get_all_tags())
}

async fn get_article_by_path(path: web::Path<String>) -> impl Responder {
    let p = decode_path(path);
    let resp = uc::get_article_content(&p).unwrap();
    HttpResponse::Ok().body(resp)
}

async fn get_asset_by_path(path: web::Path<String>) -> impl Responder {
    let p = decode_path(path);
    let bin = std::fs::read(&p).unwrap();
    HttpResponse::Ok().body(bin)
}

fn decode_path(path: web::Path<String>) -> String {
    let dec = base64::decode(&path.into_inner()).unwrap();
    String::from_utf8(dec).unwrap()
}
