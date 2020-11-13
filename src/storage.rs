use crate::domain::Metadata;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct Store {
    tags: Arc<DashMap<String, Vec<Metadata>>>, // meta by tag
    articles: Arc<DashMap<PathBuf, Metadata>>, // meta by path
}

impl Store {
    pub fn new() -> Store {
        Store {
            tags: Arc::new(DashMap::new()),
            articles: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, m: &Metadata) {
        self.articles.insert(m.path.clone(), m.clone());

        for t in &m.tags {
            match self.tags.get_mut(t) {
                Some(mut k) => {
                    k.push(m.clone());
                }
                None => {
                    self.tags.insert(t.clone(), vec![m.clone()]);
                }
            }
        }
    }

    pub fn get_all_articles(&self) -> Vec<Metadata> {
        let mut res = Vec::new();
        for k in self.articles.iter() {
            res.push(k.value().clone())
        }
        res
    }

    pub fn get_by_tag(&self, tag: &String) -> Option<Vec<Metadata>> {
        match self.tags.get(tag) {
            Some(kv) => Some(kv.value().clone()),
            None => None,
        }
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        let mut res = Vec::new();
        for k in self.tags.iter() {
            res.push(k.key().clone())
        }
        res
    }
}
