extern crate yaml_rust;
use yaml_rust::YamlLoader;

use std::fs::File;
use std::io::{prelude::*, BufReader, Lines};
use std::path::PathBuf;
use std::str;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::domain::Metadata;

static YAML_DELIM: &'static str = "---";

pub fn watch(rch: &Receiver<PathBuf>, metach: &Sender<Metadata>) {
    loop {
        match rch.recv() {
            Ok(p) => {
                let mc = Sender::clone(metach);
                thread::spawn(move || get_metadata(&p.clone(), &mc));
            }
            Err(e) => {
                println!("file handler watch err: {}", e);
                continue;
            }
        };
    }
}

fn get_metadata(e: &PathBuf, metach: &Sender<Metadata>) {
    let file = match File::open(e) {
        Ok(f) => f,
        _ => return,
    };

    let reader = BufReader::new(file);
    let yaml = get_yaml_header(reader.lines());
    let (title, tags) = yaml_to_meta(&yaml);

    let _ = metach.send(Metadata::new(e.clone(), &title, &tags));
}

fn yaml_to_meta(s: &str) -> (String, Vec<String>) {
    let docs = YamlLoader::load_from_str(s).unwrap();

    let doc = &docs[0];
    let title = doc["title"].as_str().unwrap();
    let mut tags = Vec::new();
    for t in doc["tags"].as_vec().unwrap() {
        tags.push(t.as_str().unwrap().into());
    }
    (title.into(), tags)
}

fn get_yaml_header(lines: Lines<BufReader<File>>) -> String {
    let mut header = Vec::new();
    let mut copy_yaml = false;

    for (i, line) in lines.enumerate() {
        match line {
            Ok(l) => {
                if remove_whitespace(l.as_str()) == YAML_DELIM {
                    if i == 0 {
                        copy_yaml = true;
                    } else {
                        copy_yaml = false;
                    }
                } else {
                    if copy_yaml {
                        header.extend(l.as_bytes().to_vec());
                    } else {
                        break;
                    }
                }
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }

    match str::from_utf8(&header[..]) {
        Ok(str) => str.into(),
        Err(_) => String::from(""),
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
            yaml_to_meta(&yaml),
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

        assert_eq!(get_yaml_header(BufReader::new(file).lines()), "salut");
        Ok(())
    }
}
