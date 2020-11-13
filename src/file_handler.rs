use async_std::sync::{Receiver, Sender};
use async_std::task;
use std::fs::File;
use std::io::{prelude::*, BufReader, Error as ioErr, ErrorKind, Lines, Result};
use std::path::PathBuf;
use std::str;
use yaml_rust::YamlLoader;

use crate::domain::Metadata;

static YAML_DELIM: &'static str = "---";

pub fn watch(rch: &Receiver<PathBuf>, metach: &Sender<Metadata>) {
    task::block_on(async {
        loop {
            match rch.recv().await {
                Ok(p) => {
                    let mc = Sender::clone(metach);
                    task::spawn(async move { get_metadata(&p.clone(), &mc).await });
                }
                Err(e) => {
                    println!("file handler watch err: {}", e);
                    continue;
                }
            };
        }
    });
}

async fn get_metadata(e: &PathBuf, metach: &Sender<Metadata>) -> Result<()> {
    let file = File::open(e)?;
    let reader = BufReader::new(file);
    let yaml = get_yaml_header(reader.lines())?;
    let (title, tags) = yaml_to_meta(&yaml)?;
    let _ = metach.send(Metadata::new(e.clone(), &title, &tags)).await;
    Ok(())
}

fn yaml_to_meta(s: &str) -> Result<(String, Vec<String>)> {
    let docs = match YamlLoader::load_from_str(s) {
        Ok(docs) => docs,
        Err(e) => return Err(ioErr::new(ErrorKind::NotFound, format!("{}", e))),
    };

    if docs.len() == 0 {
        return Err(ioErr::new(ErrorKind::NotFound, ""));
    }

    let doc = &docs[0];
    let title = match doc["title"].as_str() {
        Some(title) => title,
        None => return Err(ioErr::new(ErrorKind::NotFound, "no title")),
    };

    let mut tags = Vec::new();
    match doc["tags"].as_vec() {
        Some(tt) => {
            for t in tt {
                match t.as_str() {
                    Some(tag) => tags.push(tag.into()),
                    None => continue,
                }
            }
        }
        None => {}
    }

    Ok((title.into(), tags.into()))
}

fn get_yaml_header(lines: Lines<BufReader<File>>) -> Result<String> {
    let mut header = Vec::new();
    let mut copy_yaml = false;

    for (i, line) in lines.enumerate() {
        let l = line?;
        if remove_whitespace(l.as_str()) == YAML_DELIM {
            if i == 0 {
                copy_yaml = true;
            } else {
                copy_yaml = false;
            }
        } else {
            if copy_yaml {
                header.extend(format!("{}\n", l).as_bytes().to_vec());
            } else {
                break;
            }
        }
    }

    match str::from_utf8(&header[..]) {
        Ok(str) => Ok(str.into()),
        Err(_) => Err(ioErr::new(ErrorKind::NotFound, "no yaml header")),
    }
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir_in;

    #[test]
    fn yaml_to_meta_basic() -> std::io::Result<()> {
        let yaml = "title: my cool title
tags:
    - rust 
    - programming languages 
";

        assert_eq!(
            yaml_to_meta(&yaml)?,
            (
                String::from("my cool title"),
                vec!["rust".into(), "programming languages".into()]
            )
        );
        Ok(())
    }

    #[test]
    fn get_yaml_header_basic() -> std::io::Result<()> {
        let dir = tempdir_in(".")?;
        let yaml = "---
salut
---
rest
";

        let path = dir.path().join("file1");
        let mut f1 = File::create(&path)?;
        File::write_all(&mut f1, yaml.as_bytes())?;
        let file = File::open(path)?;

        assert_eq!(get_yaml_header(BufReader::new(file).lines())?, "salut\n");
        Ok(())
    }
}
