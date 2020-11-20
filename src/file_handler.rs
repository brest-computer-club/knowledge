use async_std::sync::{Receiver, Sender};
use async_std::task;
use path_abs::PathAbs;
use std::fs::File;
use std::io::{prelude::*, BufReader, Error as ioErr, ErrorKind, Lines, Result};
use std::path::PathBuf;
use std::str;
use yaml_rust::YamlLoader;

use crate::domain::{FileEvent, FileOp, Metadata, MetadataEvent};

pub fn watch(rch: &Receiver<FileEvent>, metach: &Sender<MetadataEvent>) {
    task::block_on(async {
        loop {
            match rch.recv().await {
                Ok(file_event) => {
                    if let Ok(path) = clean_path(file_event.path.clone()) {
                        match file_event.op {
                            FileOp::Create => {
                                let mc = metach.clone();
                                task::spawn(async move {
                                    let _ = handle_create(&path, &mc).await;
                                });
                            }

                            FileOp::Remove => {
                                let mc = metach.clone();
                                task::spawn(async move {
                                    mc.send(MetadataEvent::Remove(path)).await;
                                });
                            }

                            FileOp::Move => {
                                if let Some(dst) = file_event.dst {
                                    if let Ok(new_path) = clean_path(dst) {
                                        let mc = metach.clone();
                                        let old_path = path.clone();
                                        task::spawn(async move {
                                            mc.send(MetadataEvent::Move(old_path, new_path)).await;
                                        });
                                    }
                                } else {
                                }
                            }

                            FileOp::Write => {
                                let mc = metach.clone();
                                task::spawn(async move {
                                    let _ = handle_write(&path, &mc).await;
                                });
                            }
                        }
                    } else {
                        println!("invalid path {:?}", file_event.path);
                        continue;
                    }
                }
                Err(e) => {
                    println!("file handler watch err: {}", e);
                    continue;
                }
            };
        }
    });
}

fn clean_path(p: PathBuf) -> Result<PathBuf> {
    if let Ok(new_path) = PathAbs::new(p) {
        Ok(new_path.into())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "invalid path",
        ))
    }
}

async fn handle_write(p: &PathBuf, mc: &Sender<MetadataEvent>) -> Result<()> {
    let m = match get_metadata(&p).await {
        Ok(m) => m,
        Err(e) => return Err(e),
    };

    mc.send(MetadataEvent::Changed(m)).await;
    Ok(())
}

async fn handle_create(p: &PathBuf, mc: &Sender<MetadataEvent>) -> Result<()> {
    let m = match get_metadata(&p).await {
        Ok(m) => m,
        Err(e) => return Err(e),
    };

    mc.send(MetadataEvent::Create(m)).await;
    Ok(())
}

async fn get_metadata(e: &PathBuf) -> Result<Metadata> {
    let file = File::open(e)?;
    let reader = BufReader::new(file);
    let yaml = get_yaml_header(reader.lines())?;
    let (title, tags) = yaml_to_meta(&yaml)?;
    Ok(Metadata::new(e.clone(), &title, &tags))
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

static YAML_DELIM: &'static str = "---";

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
