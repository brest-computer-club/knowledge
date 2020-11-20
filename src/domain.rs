use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;
use std::path::PathBuf;

#[derive(Debug)]
pub enum MetadataEvent {
    Create(Metadata),
    Move(PathBuf, PathBuf),
    Remove(PathBuf),
    Changed(Metadata),
}

pub type Tag = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Hash, PartialOrd, Ord)]
pub struct Metadata {
    pub art: ArticleRef,
    pub tags: Vec<Tag>,
}

impl Metadata {
    pub fn new(path: PathBuf, title: &String, tags: &Vec<String>) -> Metadata {
        Metadata {
            art: ArticleRef::new(path, title),
            tags: tags.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Hash, PartialOrd, Ord)]
pub struct ArticleRef {
    pub path: PathBuf,
    pub title: String,
}

impl ArticleRef {
    pub fn new(path: PathBuf, title: &String) -> Self {
        ArticleRef {
            path,
            title: title.into(),
        }
    }
}

#[derive(Debug, Eq, Clone)]
pub enum Exp<A: Clone + Hash + Eq> {
    Sing(HashSet<A>),
    Comb(Op, Box<Exp<A>>, Box<Exp<A>>),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    And,
    Or,
}

impl<A: Hash + Eq + Clone> PartialEq for Exp<A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Exp::Sing(a), Exp::Sing(b)) => a == b,
            _ => Exp::eq(&Exp::Sing(self.reduce()), &Exp::Sing(other.reduce())),
        }
    }
}

impl<A: Hash + Eq + Clone> Exp<A> {
    pub fn reduce(&self) -> HashSet<A> {
        match self {
            Exp::Sing(a) => a.clone(),
            Exp::Comb(op, a, b) => match op {
                Op::Or => a.union(&b),
                Op::And => a.inter(&b),
            },
        }
    }

    fn inter(&self, b: &Exp<A>) -> HashSet<A> {
        match (&self, b) {
            (Exp::Sing(x), Exp::Sing(y)) => x.intersection(&y).cloned().collect(),
            _ => Exp::inter(&Exp::Sing(self.reduce()), &Exp::Sing(b.reduce())),
        }
    }

    fn union(&self, b: &Exp<A>) -> HashSet<A> {
        match (&self, b) {
            (Exp::Sing(x), Exp::Sing(y)) => x.union(&y).cloned().collect(),
            _ => Exp::union(&Exp::Sing(self.reduce()), &Exp::Sing(b.reduce())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_exp(v: Vec<u8>) -> Exp<String> {
        let a: HashSet<String> = v.clone().into_iter().map(|i| format!("{}", i)).collect();
        Exp::Sing(a)
    }
    fn to_hash(v: Vec<u8>) -> HashSet<String> {
        v.clone().into_iter().map(|i| format!("{}", i)).collect()
    }

    #[test]
    fn eq_to_self() -> std::io::Result<()> {
        let a = to_hash(vec![1, 2, 3]);
        assert_eq!(a.clone(), Exp::Sing(a.clone()).reduce());
        Ok(())
    }

    #[test]
    fn eq_to_union() -> std::io::Result<()> {
        assert_eq!(
            to_hash(vec![1, 2, 3]),
            Exp::Comb(
                Op::Or,
                Box::new(to_exp(vec![1, 3])),
                Box::new(to_exp(vec![3, 2]))
            )
            .reduce()
        );
        Ok(())
    }

    #[test]
    fn eq_to_inter() -> std::io::Result<()> {
        assert_eq!(
            to_hash(vec![3]),
            Exp::Comb(
                Op::And,
                Box::new(to_exp(vec![1, 3])),
                Box::new(to_exp(vec![3, 2]))
            )
            .reduce()
        );
        Ok(())
    }

    #[test]
    fn eq_to_union_inter_reduce() -> std::io::Result<()> {
        let a = Exp::Comb(
            Op::Or,
            Box::new(to_exp(vec![1, 3])),
            Box::new(Exp::Comb(
                Op::And,
                Box::new(to_exp(vec![3, 1, 2])),
                Box::new(to_exp(vec![3, 2, 4])),
            )),
        );

        assert_eq!(to_hash(vec![1, 2, 3]), a.reduce());
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileEvent {
    pub op: FileOp,
    pub path: PathBuf,
    pub dst: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FileOp {
    Create,
    Remove,
    Write,
    Move,
}
