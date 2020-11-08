use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Metadata {
    path: PathBuf,
    permissions: String,
}
impl Metadata {
    pub fn new(path: PathBuf, permissions: String) -> Metadata {
        Metadata { path, permissions }
    }
}
