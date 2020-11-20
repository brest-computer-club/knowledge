use async_std::sync::{channel, Receiver, Sender};
use async_std::task;
use std::collections::HashSet;
use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::thread;

use crate::domain::{Exp, FileEvent, Metadata, MetadataEvent, Op};
use crate::file_handler;
use crate::file_watcher;
use crate::metadata_handler;
use crate::storage;
use crate::tree_traverser;

pub fn build_graph_start_watcher(p: &PathBuf, store: &'static storage::Store) {
    // the file chan is used by both file_watcher & build_graph
    let (file_send, file_recv): (Sender<FileEvent>, Receiver<FileEvent>) = channel(1000);

    {
        // file_watcher
        let p_clone = p.clone();
        let file_send_clone = file_send.clone();
        thread::spawn(move || file_watcher::watch(&p_clone, &file_send_clone, &200));
    }
    {
        // build_graph
        let (meta_send, meta_recv): (Sender<MetadataEvent>, Receiver<MetadataEvent>) =
            channel(1000);
        let (dir_send, dir_recv): (Sender<PathBuf>, Receiver<PathBuf>) = channel(1000);
        let dir_send_clone = dir_send.clone();
        thread::spawn(move || tree_traverser::watch(&dir_recv, &file_send, &dir_send_clone));
        thread::spawn(move || file_handler::watch(&file_recv, &meta_send));
        thread::spawn(move || metadata_handler::watch(&meta_recv, store));
        task::block_on(async { dir_send.send(p.clone()).await });
    }
}

// todo : path is not checked, do not expose this publicly
pub fn get_article_content(p: &str) -> std::io::Result<String> {
    let content = fs::read_to_string(p)?;
    Ok(content)
}

#[derive(Debug, Clone)]
pub enum Query {
    Sing(String),
    Comb(Op, Box<Query>, Box<Query>),
}

pub fn search_by_tag(q: &Query, s: &storage::Store) -> Vec<Metadata> {
    fn new_exp(s: &storage::Store, q: &Query) -> Exp<Metadata> {
        match q {
            Query::Sing(tag) => Exp::Sing(match s.get_by_tag(tag) {
                Some(res) => res.into_iter().collect(),
                None => HashSet::new(),
            }),
            Query::Comb(op, q1, q2) => Exp::Comb(
                op.clone(),
                Box::new(new_exp(s, q1)),
                Box::new(new_exp(s, q2)),
            ),
        }
    }

    Vec::from_iter(new_exp(s, q).reduce())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Op;

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
        let s = &storage::Store::new();
        let m0 = Metadata::new(PathBuf::new(), &title(0), &vec![tag(0)]);
        let m1 = Metadata::new(PathBuf::new(), &title(1), &vec![tag(1)]);
        s.insert(&m0);
        s.insert(&m1);

        let nothing: Vec<Metadata> = vec![]; // find a way to inline it below
        assert_eq!(nothing, search_by_tag(&Query::Sing("bla".to_string()), s));
        assert_eq!(vec![m0.clone()], search_by_tag(&Query::Sing(tag(0)), s));
        assert_eq!(vec![m1.clone()], search_by_tag(&Query::Sing(tag(1)), s));
        Ok(())
    }

    #[test]
    fn search_comb() -> std::io::Result<()> {
        let s = &storage::Store::new();
        let m0 = Metadata::new(PathBuf::new(), &title(0), &vec![tag(0)]);
        let m1 = Metadata::new(PathBuf::new(), &title(1), &vec![tag(1)]);
        let m2 = Metadata::new(PathBuf::new(), &title(2), &vec![tag(0), tag(1)]);
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
            vec![m2],
            search_by_tag(
                &new_comb(Op::And, Query::Sing(tag(0)), Query::Sing(tag(1))),
                s
            )
        );
        Ok(())
    }
}
