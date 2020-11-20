use crate::domain::{ArticleRef, Tag, TaggedArticle};
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Store {
    by_tag: Arc<DashMap<Tag, Vec<ArticleRef>>>,
    by_path: Arc<DashMap<PathBuf, TaggedArticle>>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            by_tag: Arc::new(DashMap::new()),
            by_path: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, m: &TaggedArticle) {
        self.by_path.insert(m.art.path.clone(), m.clone());

        let _ = m
            .tags
            .iter()
            .map(|t| self.add_to_tag(t, &m.art))
            .collect::<Vec<()>>();
    }

    pub fn remove(&self, p: &PathBuf) {
        let remove_result = self.by_path.remove(&p.clone()).unwrap();
        let removed_meta = remove_result.1;

        let _ = removed_meta
            .tags
            .iter()
            .map(|t| {
                self.remove_from_tag(t, &removed_meta.art);
            })
            .collect::<Vec<()>>();
    }

    pub fn update_path(&self, s: PathBuf, d: PathBuf) {
        if let Some(v) = self.by_path.get(&s.clone()) {
            let new_meta = TaggedArticle::new(d.clone(), &v.art.clone().title, &v.clone().tags);
            self.by_path.insert(d.clone(), new_meta.clone());
            self.update_path_for_tags(&v.tags.clone(), &v.art.path.clone(), &new_meta.art);
            {}
        }
        self.by_path.remove(&s.clone());
    }

    pub fn update_meta(&self, m: &TaggedArticle) {
        let art = m.clone().art;

        if let Some(mut found_meta) = self.by_path.get_mut(&art.path.clone()) {
            let tags_to_insert = m.tags.iter().filter(|t| !found_meta.tags.contains(t));
            let tags_to_remove = found_meta.tags.iter().filter(|t| !m.tags.contains(t));
            let tags_in_common = m.tags.iter().filter(|t| found_meta.tags.contains(t));

            let _ = tags_in_common
                .map(|t| {
                    if let Some(mut to_update) = self.by_tag.get_mut(t) {
                        let new_tags_vec = to_update
                            .iter()
                            .map(|art_ref| {
                                if art_ref.path == art.path {
                                    &m.art
                                } else {
                                    art_ref
                                }
                            })
                            .cloned()
                            .collect::<Vec<ArticleRef>>();

                        *to_update = new_tags_vec;
                    };
                })
                .collect::<Vec<()>>();

            let _ = tags_to_remove
                .map(|t| self.remove_from_tag(t, &art))
                .collect::<Vec<()>>();

            let _ = tags_to_insert
                .map(|t| self.add_to_tag(t, &art))
                .collect::<Vec<()>>();

            *found_meta = m.clone();
        } else {
        }
    }

    pub fn get_all_articles(&self) -> Vec<TaggedArticle> {
        self.by_path.iter().map(|a| a.value().clone()).collect()
    }

    pub fn get_by_tag(&self, tag: &Tag) -> Option<Vec<ArticleRef>> {
        if let Some(kv) = self.by_tag.get(tag) {
            return Some(kv.value().clone());
        }
        None
    }

    pub fn get_all_tags(&self) -> Vec<Tag> {
        self.by_tag.iter().map(|a| a.key().clone()).collect()
    }

    //
    // privates
    //

    fn add_to_tag(&self, t: &Tag, art: &ArticleRef) {
        match self.by_tag.get_mut(t) {
            Some(mut k) => {
                k.push(art.clone());
            }
            None => {
                self.by_tag.insert(t.clone(), vec![art.clone()]);
            }
        }
    }

    fn remove_from_tag(&self, t: &Tag, art: &ArticleRef) {
        self.by_tag.alter(t, |_, v| {
            v.iter().filter(|a| a.path != art.path).cloned().collect()
        });

        self.by_tag.remove_if(t, |_, tag_vec| tag_vec.len() == 0);
    }

    fn update_path_for_tags(
        &self,
        tags_to_update: &Vec<String>,
        old_path: &PathBuf,
        new_art: &ArticleRef,
    ) {
        tags_to_update
            .iter()
            .map(|t| {
                self.by_tag.alter(t, |_, v| {
                    vec![new_art.clone()]
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

    fn art(i: i32) -> ArticleRef {
        ArticleRef {
            path: format!("path_{}", i).into(),
            title: format!("title_{}", i),
        }
    }

    fn tags(t: Vec<i32>) -> Vec<Tag> {
        t.iter().map(|i| format!("tag_{}", i)).collect()
    }

    #[test]
    fn new() -> std::io::Result<()> {
        let s = Store::new();
        {
            // initialized empty
            assert_eq!(0, s.by_path.len());
            assert_eq!(0, s.by_tag.len());
        }
        Ok(())
    }

    #[test]
    fn update_meta() -> std::io::Result<()> {
        let t1 = "tag_1".to_string();
        let t2 = "tag_2".to_string();
        let t3 = "tag_3".to_string();

        let old = TaggedArticle {
            art: art(1),
            tags: tags(vec![1, 2]),
        };

        let mut new = old.clone();
        new.tags = tags(vec![3, 2]);
        new.art.title = "title_2".to_string();

        let s = Store::new();
        s.insert(&old);

        s.update_meta(&new.clone());

        {
            // articles update
            assert_eq!(new, *s.by_path.get(&old.art.path).unwrap());
        }
        {
            // tags update

            assert!(s.by_tag.get(&t1).is_none());
            assert!(s.by_tag.get(&t2).is_some());
            assert!(s.by_tag.get(&t3).is_some());

            let _ = s
                .by_tag
                .iter()
                .flat_map(|t| {
                    t.iter()
                        .filter(|t| t.path == new.art.path)
                        .cloned()
                        .collect::<Vec<ArticleRef>>()
                })
                .map(|m| assert_eq!(new.art.clone(), m))
                .collect::<Vec<()>>();
        }

        Ok(())
    }

    #[test]
    fn insert() -> std::io::Result<()> {
        let m1 = TaggedArticle {
            art: art(1),
            tags: tags(vec![1, 2]),
        };
        let m2 = TaggedArticle {
            art: art(2),
            tags: tags(vec![3, 2]),
        };

        let s = Store::new();

        {
            // simple insertion works
            s.insert(&m1);
            assert_eq!(1, s.by_path.len());
            assert_eq!(2, s.by_tag.len());
        }

        {
            // duplicate tags (tag2) should append metadata
            s.insert(&m2);
            assert_eq!(2, s.by_path.len());
            assert_eq!(3, s.by_tag.len());
        }

        {
            // articles aren't altered at insertion
            assert_eq!(m1, *s.by_path.get(&m1.art.path).unwrap());
            assert_eq!(m2, *s.by_path.get(&m2.art.path).unwrap());
            assert_ne!(m1, *s.by_path.get(&m2.art.path).unwrap());
        }

        Ok(())
    }

    fn count_path_found_all_tags(s: &Store, p: &PathBuf) -> usize {
        s.by_tag
            .iter()
            .map(|t| {
                t.iter()
                    .filter(|t| &t.path == p)
                    .cloned()
                    .collect::<Vec<ArticleRef>>()
                    .len()
            })
            .collect::<Vec<usize>>()
            .iter()
            .sum()
    }

    #[test]
    fn remove() -> std::io::Result<()> {
        let m1 = TaggedArticle {
            art: art(1),
            tags: tags(vec![1, 2]),
        };
        let m2 = TaggedArticle {
            art: art(2),
            tags: tags(vec![2, 3]),
        };

        let s = Store::new();
        s.insert(&m1.clone());
        s.insert(&m2);

        let to_remove = m1.art.path.clone();
        s.remove(&to_remove);

        {
            //articles
            assert!(s.by_path.get(&m1.art.path).is_none());
            assert!(s.by_path.get(&m2.art.path).is_some());
        }

        {
            //tags
            assert_eq!(0, count_path_found_all_tags(&s.clone(), &m1.art.path));
            assert_eq!(2, count_path_found_all_tags(&s.clone(), &m2.art.path));
        }
        Ok(())
    }

    #[test]
    fn update_path() -> std::io::Result<()> {
        let art1 = art(1);
        let art2 = art(2);
        let m1 = TaggedArticle {
            art: art1.clone(),
            tags: tags(vec![1, 2]),
        };
        let m2 = TaggedArticle {
            art: art2,
            tags: tags(vec![2, 3]),
        };

        let new_path: PathBuf = "3".into();
        let mut new_meta = art1.clone();
        new_meta.path = new_path.clone();

        let s = Store::new();

        {
            // preconditions
            s.insert(&m1);
            s.insert(&m2);

            assert_eq!(2, s.by_path.len());
            assert_eq!(3, s.by_tag.len());
        }

        s.update_path(m1.art.path.clone(), new_path.clone());

        {
            // check articles update
            assert!(s.by_path.get(&m1.art.path).is_none());
            assert!(s.by_path.get(&m2.art.path).is_some());
            assert!(s.by_path.get(&new_path).is_some());
        }
        {
            // check tags update

            // m1 path updated for tag1 (it's alone)
            assert_eq!(Some(&new_meta), s.by_tag.get("tag_1").unwrap().first());

            // m1 path updated for tag2
            assert_eq!(
                Some(&&new_meta),
                s.by_tag
                    .get("tag_2")
                    .unwrap()
                    .iter()
                    .filter(|t| t.title == art1.title)
                    .collect::<Vec<&ArticleRef>>()
                    .first()
            );

            // m1 not added to other tag
            assert!(s
                .by_tag
                .get("tag_3")
                .unwrap()
                .iter()
                .filter(|t| t.title == art1.title)
                .collect::<Vec<&ArticleRef>>()
                .first()
                .is_none());
        }

        Ok(())
    }
}
