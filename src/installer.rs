use crate::config::Configuration;
use crate::updater::Updater;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

pub struct Installer {
    cfg: Configuration,
}

impl Installer {
    pub fn new(cfg: Configuration) -> Self {
        Self { cfg }
    }

    pub fn clean_install(
        &self,
        status_tx: Sender<String>,
        tx: Sender<u32>,
        update_tx: Sender<u32>,
        progress_tx: Sender<f64>,
        ctx: egui::Context,
    ) {
        let game_dl = self.cfg.wow.clone();
        let path = self.cfg.path.clone();
        let cfg = self.cfg.clone();

        if !PathBuf::from(&cfg.path).exists() {
            std::fs::create_dir_all(&cfg.path).unwrap();
        }

        tokio::spawn(async move {
            tx.send(1).unwrap();
            ctx.request_repaint();

            status_tx
                .send("Downloading game files...".to_string())
                .unwrap();

            //  send progress of download to status_ts
            println!("Downloading {:?}", game_dl);
            let mut game_files = match reqwest::get(game_dl).await {
                Ok(res) => res,
                Err(_) => {
                    tx.send(0).unwrap();
                    ctx.request_repaint();
                    return;
                }
            };

            let mut file = std::fs::File::create("game.zip").unwrap();

            // yeah wow is a little big raizo..
            let mut downloaded = 0;
            let total_size = game_files.content_length().unwrap_or(0);

            while let Some(chunk) = game_files.chunk().await.unwrap() {
                downloaded += chunk.len();
                file.write(&chunk).unwrap();

                let progress = downloaded as f64 / total_size as f64;

                progress_tx.send(progress).unwrap();
            }

            println!("Done downloading");

            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open("game.zip")
                .unwrap();

            status_tx
                .send("Installing game files...".to_string())
                .unwrap();

            let mut zip = zip::ZipArchive::new(file).unwrap();
            for i in 0..zip.len() {
                let mut file = zip.by_index(i).unwrap();
                let file_name = file.name();

                let mut dest_file_path = PathBuf::from(&path);
                dest_file_path.push(file_name);

                let parent = dest_file_path.parent().unwrap();
                if !parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }

                if file.is_dir() {
                    std::fs::create_dir_all(dest_file_path).unwrap();
                    continue;
                }

                let mut dest_file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(dest_file_path)
                    .unwrap();

                io::copy(&mut file, &mut dest_file).unwrap();
            }

            status_tx.send("Starting Updater...".to_string()).unwrap();

            tx.send(0).unwrap();
            ctx.request_repaint();

            Updater::new(cfg).check(status_tx, update_tx, ctx);
        });
    }
}
