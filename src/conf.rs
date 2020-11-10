use crate::storage;

lazy_static! {
    pub static ref STORE: storage::Store = storage::Store::new();
}
