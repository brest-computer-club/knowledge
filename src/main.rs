use std::env;
use std::thread;

mod api;
mod uc;
mod walker;

fn main() -> std::io::Result<()> {
    {
        let path = env::current_dir()?;

        thread::spawn(move || {
            let i = uc::Interactor {
                visitor: &uc::FileVisitor::new(),
            };

            i.build_graph(&path.as_path(), &walker::W);
        });
    }

    api::start_server(8080)
}
