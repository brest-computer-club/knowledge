use async_std::sync::{channel, Receiver, Sender};
use async_std::task;
use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::thread;
use storage::Store;

use crate::domain::{ArtRef, Exp, FileEvent, MetadataEvent, Op};
use crate::file_handler;
use crate::file_watcher;
use crate::metadata_handler;
use crate::storage;
use crate::tree_traverser;

pub fn build_graph_start_watcher(p: &PathBuf, store: &'static Store) {
    // the file chan is used by both file_watcher & build_graph
    let (file_send, file_rcv): (Sender<FileEvent>, Receiver<FileEvent>) = channel(100);

    {
        // file_watcher
        let p_ = p.clone();
        let file_send_ = file_send.clone();
        thread::spawn(move || file_watcher::watch(&p_, &file_send_, 200));
    }
    {
        // build_graph
        let (meta_send, meta_rcv): (Sender<MetadataEvent>, Receiver<MetadataEvent>) = channel(100);
        let (dir_send, dir_rcv): (Sender<PathBuf>, Receiver<PathBuf>) = channel(100);
        let dir_send_ = dir_send.clone();
        thread::spawn(move || tree_traverser::watch(&dir_rcv, &dir_send, &file_send));
        thread::spawn(move || file_handler::watch(&file_rcv, &meta_send));
        thread::spawn(move || metadata_handler::watch(&meta_rcv, store));
        task::block_on(async { dir_send_.send(p.clone()).await });
    }
}

#[derive(Debug, Clone)]
pub enum Query {
    Sing(String),
    Comb(Op, Box<Query>, Box<Query>),
}

pub fn search_by_tag(q: &Query, s: &Store) -> Vec<ArtRef> {
    fn new_exp(s: &Store, q: &Query) -> Exp<ArtRef> {
        match q {
            Query::Sing(tag) => Exp::Sing(s.get_by_tag(tag).into_iter().collect()),
            Query::Comb(op, q1, q2) => Exp::Comb(
                op.clone(),
                Box::new(new_exp(s, q1)),
                Box::new(new_exp(s, q2)),
            ),
        }
    }

    Vec::from_iter(new_exp(s, q).reduce())
}

// todo : path is not checked, do not expose this publicly
pub fn get_article_content(p: &str) -> std::io::Result<String> {
    fs::read_to_string(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Op, TaggedArticle};

    // helpers
    fn new_comb(op: Op, q1: Query, q2: Query) -> Query {
        Query::Comb(op, Box::new(q1), Box::new(q2))
    }

    fn title(i: u8) -> String {
        format!("title_{}", i)
    }
    fn tag(i: u8) -> String {
        format!("tag_{}", i)
    }

    #[test]
    fn search_sing() -> std::io::Result<()> {
        let s = &Store::new();
        let art0 = ArtRef::new(PathBuf::new(), &title(0));
        let art1 = ArtRef::new(PathBuf::new(), &title(1));
        let m0 = &TaggedArticle::new_from_art(&art0, &vec![tag(0)]);
        let m1 = &TaggedArticle::new_from_art(&art1, &vec![tag(1)]);
        s.insert(m0);
        s.insert(m1);

        let nothing: Vec<ArtRef> = vec![]; // find a way to inline it below
        assert_eq!(nothing, search_by_tag(&Query::Sing("bla".to_string()), s));
        assert_eq!(vec![art0], search_by_tag(&Query::Sing(tag(0)), s));
        assert_eq!(vec![art1], search_by_tag(&Query::Sing(tag(1)), s));
        Ok(())
    }

    #[test]
    fn search_comb() -> std::io::Result<()> {
        let s = &Store::new();
        let m0 = TaggedArticle::new(PathBuf::new(), &title(0), &vec![tag(0)]);
        let m1 = TaggedArticle::new(PathBuf::new(), &title(1), &vec![tag(1)]);

        let art2 = ArtRef::new(PathBuf::new(), &title(2));
        let m2 = TaggedArticle::new_from_art(&art2, &vec![tag(0), tag(1)]);
        s.insert(&m0);
        s.insert(&m1);
        s.insert(&m2);

        assert_eq!(
            vec![m0.clone(), m1.clone(), m2.clone()].sort(),
            search_by_tag(
                &new_comb(Op::Or, Query::Sing(tag(0)), Query::Sing(tag(1))),
                s
            )
            .sort()
        );

        assert_eq!(
            vec![art2],
            search_by_tag(
                &new_comb(Op::And, Query::Sing(tag(0)), Query::Sing(tag(1))),
                s
            )
        );
        Ok(())
    }
}
