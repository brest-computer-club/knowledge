use clap::{App, Arg, ArgMatches};
use lazy_static::lazy_static;
use rand::Rng;
use simple_logger::SimpleLogger;
use std::{env, thread};
use std::{io, path::PathBuf};
use webbrowser;

mod api;
mod domain;
mod file_handler;
mod file_watcher;
mod metadata_handler;
mod storage;
mod tree_traverser;
mod uc;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let mm = cli_setup();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .init()
        .unwrap();

    lazy_static! {
        static ref STORE: storage::Store = storage::Store::new();
    }

    {
        let f = get_folder(&mm)?;
        thread::spawn(move || uc::build_graph_start_watcher(&f, &STORE));
    }

    {
        let bind_addr = format!("localhost:{}", get_listen_port(&mm));
        let dev_mode = in_dev_mode(&mm);
        if !dev_mode {
            let url = format!("http://{}", bind_addr.clone());
            thread::spawn(move || webbrowser::open(&url));
        }

        api::server(&bind_addr, &STORE, &dev_mode)?.await
    }
}

fn cli_setup() -> ArgMatches {
    App::new("Knowledge")
        .version("0.0.2")
        .author("Brest Computer Club <brest-computer-club@protonmail.com>")
        .about("transform your markdown file into a knowledge base")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .about("the port used by the app")
                .takes_value(true),
        )
        .arg(
            Arg::new("folder")
                .short('f')
                .long("folder")
                .about("the root folder")
                .takes_value(true),
        )
        .arg(
            Arg::new("dev_mode")
                .short('d')
                .long("dev")
                .about("run in dev mode")
                .takes_value(false),
        )
        .get_matches()
}

fn get_folder(mm: &ArgMatches) -> io::Result<PathBuf> {
    let f: String = mm.value_of_t("folder").unwrap_or("".to_string());
    if f == "" {
        return env::current_dir();
    }

    Ok(PathBuf::from(f))
}

fn get_listen_port(mm: &ArgMatches) -> u16 {
    if in_dev_mode(mm) {
        return 8080;
    }

    mm.value_of_t("port")
        .unwrap_or(rand::thread_rng().gen_range(3000, 10000))
}

fn in_dev_mode(mm: &ArgMatches) -> bool {
    mm.is_present("dev_mode")
}
