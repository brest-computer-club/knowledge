use crate::domain;
use crate::storage;
use crate::uc;
use crate::uc::Query;
use actix_cors::Cors;
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

pub fn server(
    address: &str,
    store: &'static storage::Store,
    dev_mode: bool,
) -> Result<Server, std::io::Error> {
    let addr = address.to_string();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(set_cors(&addr.clone(), dev_mode))
            .data(store.clone())
            .configure(static_routes)
            .configure(back_routes)
    })
    .bind(address)?
    .run();

    println!("listening on : {}", address);
    Ok(server)
}

fn set_cors(address: &str, dev_mode: bool) -> Cors {
    let bind_addr = &format!("http://{}", address)[..];
    if dev_mode {
        return Cors::default()
            .allowed_origin("http://localhost:8000")
            .allowed_origin(bind_addr)
            .allowed_methods(vec!["GET", "POST"]);
    }
    Cors::default()
        .allowed_methods(vec!["GET", "POST"])
        .allowed_origin(bind_addr)
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
            .route("/tags/{tag}", web::get().to(get_by_tag))
            .route("/search-by-tags", web::post().to(search_by_tag))
            .route("/articles", web::get().to(get_all_articles))
            .route("/articles/{path}", web::get().to(get_article_by_path))
            .route("/assets/{path}", web::get().to(get_asset_by_path)),
    );
}

//
// frontend routes
//
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

//
// backend routes
//
async fn get_by_tag(store: web::Data<storage::Store>, tag: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().json(store.get_by_tag(&tag.into_inner()))
}

async fn get_all_articles(store: web::Data<storage::Store>) -> impl Responder {
    HttpResponse::Ok().json(store.get_all_articles())
}

async fn get_all_tags(store: web::Data<storage::Store>) -> impl Responder {
    HttpResponse::Ok().json(store.get_all_tags())
}
use actix_web::http::header::ContentType;

async fn get_article_by_path(path: web::Path<String>) -> impl Responder {
    let p = decode_path(path);
    let resp = uc::get_article_content(&p).unwrap();
    let mut builder = HttpResponse::Ok();
    builder.set(ContentType::plaintext());
    builder.body(resp)
}

async fn get_asset_by_path(path: web::Path<String>) -> impl Responder {
    let p = decode_path(path);
    let bin = std::fs::read(&p).unwrap();
    HttpResponse::Ok().body(bin)
}

async fn search_by_tag(
    store: web::Data<storage::Store>,
    json_query: web::Json<JsonQuery>,
) -> impl Responder {
    HttpResponse::Ok().json(uc::search_by_tag(&JsonQuery::to_uc(&json_query), &store))
}

fn decode_path(path: web::Path<String>) -> String {
    let dec = base64::decode(&path.into_inner()).unwrap();
    String::from_utf8(dec).unwrap()
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum JsonQuery {
    Sing {
        val: String,
    },
    Comb {
        op: domain::Op,
        qa: Box<JsonQuery>,
        qb: Box<JsonQuery>,
    },
}

impl JsonQuery {
    fn to_uc(jq: &JsonQuery) -> Query {
        match &jq {
            JsonQuery::Sing { val } => Query::Sing(val.clone()),
            JsonQuery::Comb { op, qa, qb } => Query::Comb(
                op.clone(),
                Box::new(JsonQuery::to_uc(qa)),
                Box::new(JsonQuery::to_uc(qb)),
            ),
        }
    }
}
