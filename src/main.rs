use std::env;
use std::thread;

mod api;
mod metadata_extractor;
mod metadata_handler;
mod tree_traverser;
mod uc;

fn main() -> std::io::Result<()> {
    {
        let p = env::current_dir()?;
        thread::spawn(move || uc::build_graph(&p));
    }

    api::start_server(8080)
}
