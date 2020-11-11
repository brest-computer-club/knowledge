use crate::domain::Metadata;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Store {
    meta_index: Arc<DashMap<String, Vec<Metadata>>>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            meta_index: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, m: &Metadata) {
        for t in &m.tags {
            match self.meta_index.get_mut(t) {
                Some(mut k) => {
                    k.push(m.clone());
                }
                None => {
                    self.meta_index.insert(t.clone(), vec![m.clone()]);
                }
            }
        }
    }

    pub fn get_by_tag(&self, tag: &String) -> Option<Vec<Metadata>> {
        match self.meta_index.get(tag) {
            Some(kv) => Some(kv.value().clone()),
            None => None,
        }
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        let mut res = Vec::new();
        for k in self.meta_index.iter() {
            res.push(k.key().clone())
        }
        res
    }
}
