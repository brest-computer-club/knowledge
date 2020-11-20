use crate::domain::Metadata;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;

// TODO : for tags, keep only PathBuf & title
#[derive(Clone, Debug)]
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

        let _ = m
            .tags
            .iter()
            .map(|t| self.add_to_tag(t, m))
            .collect::<Vec<()>>();
    }

    pub fn remove(&self, p: &PathBuf) {
        let remove_result = self.articles.remove(&p.clone()).unwrap();
        let removed_meta = remove_result.1;

        let _ = removed_meta
            .tags
            .iter()
            .map(|t| {
                self.remove_from_tag(t, &removed_meta);
            })
            .collect::<Vec<()>>();
    }

    pub fn update_path(&self, s: PathBuf, d: PathBuf) {
        if let Some(v) = self.articles.get(&s.clone()) {
            let new_meta = Metadata {
                path: d.clone(),
                tags: v.clone().tags,
                title: v.clone().title,
            };

            self.articles.insert(d.clone(), new_meta.clone());
            self.update_path_for_tags(&v.tags.clone(), &v.path.clone(), new_meta.clone());
            {}
        }
        self.articles.remove(&s.clone());
    }

    pub fn update_meta(&self, m: &Metadata) {
        if let Some(mut stored_article) = self.articles.get_mut(&m.path.clone()) {
            let tags_to_insert = m.tags.iter().filter(|t| !stored_article.tags.contains(t));
            let tags_to_remove = stored_article.tags.iter().filter(|t| !m.tags.contains(t));
            let tags_in_common = m.tags.iter().filter(|t| stored_article.tags.contains(t));

            let _ = tags_in_common
                .map(|t| {
                    if let Some(mut to_update) = self.tags.get_mut(t) {
                        let new_vec = to_update
                            .iter()
                            .map(|meta| if meta.path == m.path { m } else { meta })
                            .cloned()
                            .collect::<Vec<Metadata>>();

                        *to_update = new_vec;
                    };
                })
                .collect::<Vec<()>>();

            let _ = tags_to_remove
                .map(|t| self.remove_from_tag(t, m))
                .collect::<Vec<()>>();

            let _ = tags_to_insert
                .map(|t| self.add_to_tag(t, m))
                .collect::<Vec<()>>();

            *stored_article = m.clone();
        } else {
        }
    }

    pub fn get_all_articles(&self) -> Vec<Metadata> {
        self.articles.iter().map(|a| a.value().clone()).collect()
    }

    pub fn get_by_tag(&self, tag: &String) -> Option<Vec<Metadata>> {
        if let Some(kv) = self.tags.get(tag) {
            return Some(kv.value().clone());
        }
        None
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        self.tags.iter().map(|a| a.key().clone()).collect()
    }

    //
    // privates
    //

    fn add_to_tag(&self, t: &String, m: &Metadata) {
        match self.tags.get_mut(t) {
            Some(mut k) => {
                k.push(m.clone());
            }
            None => {
                self.tags.insert(t.clone(), vec![m.clone()]);
            }
        }
    }

    fn remove_from_tag(&self, t: &String, m: &Metadata) {
        self.tags.alter(t, |_, v| {
            v.iter().filter(|a| a.path != m.path).cloned().collect()
        });

        // remove the key if it's empty
        self.tags.remove_if(t, |_, tag_vec| tag_vec.len() == 0);
    }

    fn update_path_for_tags(
        &self,
        tags_to_update: &Vec<String>,
        old_path: &PathBuf,
        new_meta: Metadata,
    ) {
        tags_to_update
            .iter()
            .map(|t| {
                self.tags.alter(t, |_, v| {
                    vec![new_meta.clone()]
                        .iter()
                        .chain(v.iter().filter(|m| &m.path != &old_path.clone()))
                        .cloned()
                        .collect()
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() -> std::io::Result<()> {
        let s = Store::new();
        {
            // initialized empty
            assert_eq!(0, s.articles.len());
            assert_eq!(0, s.tags.len());
        }
        Ok(())
    }

    #[test]
    fn update_meta() -> std::io::Result<()> {
        let t1 = "tag1".to_string();
        let t2 = "tag2".to_string();
        let t3 = "tag3".to_string();

        let old = Metadata {
            path: "1".into(),
            tags: vec![t1.clone(), t2.clone()],
            title: "title1".to_string(),
        };

        let mut new = old.clone();
        new.tags = vec![t3.clone(), t2.clone()];
        new.title = "title2".to_string();

        let s = Store::new();
        s.insert(&old);

        s.update_meta(&new.clone());

        {
            // articles update
            assert_eq!(new, *s.articles.get(&old.path).unwrap());
        }
        {
            // tags update

            assert!(s.tags.get(&t1).is_none());
            assert!(s.tags.get(&t2).is_some());
            assert!(s.tags.get(&t3).is_some());

            let _ = s
                .tags
                .iter()
                .flat_map(|t| {
                    t.iter()
                        .filter(|t| t.path == new.path)
                        .cloned()
                        .collect::<Vec<Metadata>>()
                })
                .map(|m| assert_eq!(new.clone(), m))
                .collect::<Vec<()>>();
        }

        Ok(())
    }

    #[test]
    fn insert() -> std::io::Result<()> {
        let m1 = Metadata {
            path: "1".into(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            title: "title1".to_string(),
        };
        let m2 = Metadata {
            path: "2".into(),
            tags: vec!["tag2".to_string(), "tag3".to_string()],
            title: "title2".to_string(),
        };

        let s = Store::new();

        {
            // simple insertion works
            s.insert(&m1);
            assert_eq!(1, s.articles.len());
            assert_eq!(2, s.tags.len());
        }

        {
            // duplicate tags (tag2) should append metadata
            s.insert(&m2);
            assert_eq!(2, s.articles.len());
            assert_eq!(3, s.tags.len());
        }

        {
            // articles aren't altered at insertion
            assert_eq!(m1, *s.articles.get(&m1.path).unwrap());
            assert_eq!(m2, *s.articles.get(&m2.path).unwrap());
            assert_ne!(m1, *s.articles.get(&m2.path).unwrap());
        }

        Ok(())
    }

    fn count_path_found_all_tags(s: &Store, p: &PathBuf) -> usize {
        s.tags
            .iter()
            .map(|t| {
                t.iter()
                    .filter(|t| &t.path == p)
                    .cloned()
                    .collect::<Vec<Metadata>>()
                    .len()
            })
            .collect::<Vec<usize>>()
            .iter()
            .sum()
    }

    #[test]
    fn remove() -> std::io::Result<()> {
        let m1 = Metadata {
            path: "1".into(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            title: "title1".to_string(),
        };
        let m2 = Metadata {
            path: "2".into(),
            tags: vec!["tag2".to_string(), "tag3".to_string()],
            title: "title2".to_string(),
        };

        let s = Store::new();
        s.insert(&m1.clone());
        s.insert(&m2);

        let to_remove = m1.path.clone();
        s.remove(&to_remove);

        {
            //articles
            assert!(s.articles.get(&m1.path).is_none());
            assert!(s.articles.get(&m2.path).is_some());
        }

        {
            //tags
            assert_eq!(0, count_path_found_all_tags(&s.clone(), &m1.path));
            assert_eq!(2, count_path_found_all_tags(&s.clone(), &m2.path));
        }
        Ok(())
    }

    #[test]
    fn update_path() -> std::io::Result<()> {
        let m1 = Metadata {
            path: "1".into(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            title: "title1".to_string(),
        };
        let m2 = Metadata {
            path: "2".into(),
            tags: vec!["tag2".to_string(), "tag3".to_string()],
            title: "title2".to_string(),
        };
        let new_path: PathBuf = "3".into();
        let mut new_meta = m1.clone();
        new_meta.path = new_path.clone();

        let s = Store::new();

        {
            // preconditions
            s.insert(&m1);
            s.insert(&m2);

            assert_eq!(2, s.articles.len());
            assert_eq!(3, s.tags.len());
        }

        s.update_path(m1.path.clone(), new_path.clone());

        {
            // check articles update
            assert!(s.articles.get(&m1.path).is_none());
            assert!(s.articles.get(&m2.path).is_some());
            assert!(s.articles.get(&new_path).is_some());
        }
        {
            // check tags update

            // m1 path updated for tag1 (it's alone)
            assert_eq!(Some(&new_meta), s.tags.get("tag1").unwrap().first());

            // m1 path updated for tag2
            assert_eq!(
                Some(&&new_meta),
                s.tags
                    .get("tag2")
                    .unwrap()
                    .iter()
                    .filter(|t| t.title == m1.title)
                    .collect::<Vec<&Metadata>>()
                    .first()
            );

            // m1 not added to other tag
            assert!(s
                .tags
                .get("tag3")
                .unwrap()
                .iter()
                .filter(|t| t.title == m1.title)
                .collect::<Vec<&Metadata>>()
                .first()
                .is_none());
        }

        Ok(())
    }
}
