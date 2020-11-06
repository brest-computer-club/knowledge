use crate::walker::Walker;

pub fn build(w: &impl Walker) {
    let _ = w.get_root();
}
