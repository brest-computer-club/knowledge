use std::thread;

mod api;
mod uc;
mod walker;

fn main() -> std::io::Result<()> {
    thread::spawn(|| {
        let w = &walker::local::W;
        let _ = uc::graph::build(w);
    });

    api::server::start_server(8080)
}
