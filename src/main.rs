use clap::{App, Arg, ArgMatches};
use lazy_static::lazy_static;
use std::io;

use rand::Rng;
use std::{env, thread};
use webbrowser;

mod api;
mod domain;
mod file_handler;
mod metadata_handler;
mod storage;
mod tree_traverser;
mod uc;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let matches = cli_setup();

    lazy_static! {
        static ref STORE: storage::Store = storage::Store::new();
    }

    {
        // walk the current folder and build the store
        let p = env::current_dir()?;
        thread::spawn(move || uc::build_graph(&p, &STORE));
    }

    {
        let bind_addr = format!("127.0.0.1:{}", get_listen_port(&matches));
        if !in_dev_mode(&matches) {
            // launch the webbrowser
            let url = format!("http://{}", bind_addr.clone());
            thread::spawn(move || webbrowser::open(&url));
        }

        // start the API
        api::server(&bind_addr, &STORE, &false)?.await
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
            Arg::new("dev_mode")
                .short('d')
                .long("dev")
                .about("run in dev mode")
                .takes_value(false),
        )
        .get_matches()
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
