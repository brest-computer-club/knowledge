use crate::walker::Walker;

use crate::domain::tree;

pub fn build(w: &impl Walker) {
    match w.get_root() {
        Ok(r) => w.walk_tree(&r.as_path(), &tree::visit),
        _ => (),
    };

    println!("done")
}
