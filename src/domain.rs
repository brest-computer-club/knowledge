use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Metadata {
    path: PathBuf,
    title: String,
    pub tags: Vec<String>,
}

impl Metadata {
    pub fn new(path: PathBuf, title: &String, tags: &Vec<String>) -> Metadata {
        Metadata {
            path,
            title: title.clone(),
            tags: tags.clone(),
        }
    }
}
