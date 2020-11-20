use crate::domain::{FileEvent, FileOp};
use async_std::{sync::Sender, task};
use log::info;
use notify::{
    watcher,
    DebouncedEvent::{Create, Remove, Rename, Write},
    RecursiveMode::Recursive,
    Watcher,
};
use std::{io, path::PathBuf, sync::mpsc::channel, time::Duration};

pub fn watch(
    root_path: &PathBuf,
    send_chan: &Sender<FileEvent>,
    debounce: u64,
) -> Result<(), io::Error> {
    info!("watching for file changes in {:?}", root_path.clone());

    let (tx, rx) = channel();
    let mut w = match watcher(tx, Duration::from_millis(debounce)) {
        Ok(w) => w,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                format!("{:?}", e),
            ))
        }
    };

    let _ = w.watch(root_path, Recursive);

    for e in rx.iter() {
        match e {
            Create(p) => {
                info!("adding new file {:?}", p);
                let s = send_chan.clone();
                task::spawn(async move {
                    s.send(FileEvent {
                        op: FileOp::Create,
                        path: p,
                        dst: None,
                    })
                    .await
                });
            }
            Write(p) => {
                info!("checking for updates in {:?}", p);
                let s = send_chan.clone();
                task::spawn(async move {
                    s.send(FileEvent {
                        op: FileOp::Write,
                        path: p,
                        dst: None,
                    })
                    .await;
                });
            }
            Rename(src, dst) => {
                info!("updating path from {:?} to {:?}", src, dst);
                let s = send_chan.clone();
                task::spawn(async move {
                    s.send(FileEvent {
                        op: FileOp::Move,
                        path: src,
                        dst: Some(dst),
                    })
                    .await;
                });
            }
            Remove(p) => {
                info!("removing file {:?}", p);
                let s = send_chan.clone();
                task::spawn(async move {
                    s.send(FileEvent {
                        op: FileOp::Remove,
                        path: p,
                        dst: None,
                    })
                    .await;
                });
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::sync::{channel, Receiver, Sender};
    use std::fs;
    use std::thread;
    use tempfile::tempdir;

    #[async_std::test]
    async fn inotifications() -> std::io::Result<()> {
        let dir = tempdir()?;
        let file_1 = dir.path().join("file1");
        let file_2 = dir.path().join("file2");

        let (tx, rx): (Sender<FileEvent>, Receiver<FileEvent>) = channel(1000);

        let debounce: u64 = 1;
        thread::spawn(move || watch(&dir.path().to_path_buf(), &tx.clone(), debounce));
        thread::sleep(std::time::Duration::from_millis(debounce + 10)); // looks fragile ?

        // Create
        fs::File::create(&file_1.clone())?;
        assert_eq!(
            Ok(FileEvent {
                op: FileOp::Create,
                path: file_1.clone(),
                dst: None
            }),
            rx.recv().await
        );

        // Write
        fs::write(file_1.clone(), "data")?;
        assert_eq!(
            Ok(FileEvent {
                op: FileOp::Write,
                path: file_1.clone(),
                dst: None
            }),
            rx.recv().await
        );

        // Move
        fs::rename(file_1.clone(), file_2.clone())?;
        assert_eq!(
            Ok(FileEvent {
                op: FileOp::Move,
                path: file_1.clone(),
                dst: Some(file_2.clone())
            }),
            rx.recv().await
        );

        // Delete
        fs::remove_file(file_2.clone())?;
        assert_eq!(
            Ok(FileEvent {
                op: FileOp::Remove,
                path: file_2.clone(),
                dst: None
            }),
            rx.recv().await
        );

        Ok(())
    }
}
