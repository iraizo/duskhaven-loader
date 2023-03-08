use std::{
    fs::{self, DirEntry, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::mpsc::Sender,
};

use crate::config::Configuration;

pub struct Updater {
    cfg: Configuration,
}

impl Updater {
    pub fn new(cfg: Configuration) -> Self {
        Self { cfg }
    }

    pub fn check(&self, status_tx: Sender<String>, tx: Sender<u32>, ctx: egui::Context) {
        let files = self.cfg.files.clone();
        let path = self.cfg.path.clone();
        let list = self.cfg.realmlist.clone();
        tokio::spawn(async move {
            tx.send(1).unwrap();
            ctx.request_repaint();
            status_tx.send("Updating..".to_owned()).unwrap();

            // find the data folder
            let dir: Vec<_> = fs::read_dir(&path)
                .unwrap()
                .filter(|entry| {
                    if let Ok(entry) = entry {
                        return entry.file_name() == "Data";
                    } else {
                        false
                    }
                })
                .collect();

            // find the lang specific folder
            let lang_path: Vec<_> = fs::read_dir(dir[0].as_ref().clone().unwrap().path())
                .unwrap()
                .filter(|entry| entry.as_ref().unwrap().file_type().unwrap().is_dir())
                .collect();

            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .append(false)
                .open(format!(
                    "{}\\realmlist.wtf",
                    lang_path[0].as_ref().unwrap().path().display()
                ))
                .unwrap();

            file.write_all(list.as_bytes()).unwrap();

            /*   for entry in dir {
                let entry = entry.unwrap();
                let mut file = OpenOptions::new()
                    .write(true)
                    .open(format!("{}\\Data\\realmlist.wtf", entry.path().display()))
                    .unwrap();

                file.write_all(list.as_bytes()).unwrap();
            } */

            for file in &files {
                let res = reqwest::get(&file.url).await.unwrap();

                if res
                    .headers()
                    .get("etag")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace(r#"""#, "")
                    != file.etag
                {
                    //Resource is outdated
                    status_tx
                        .send(format!("Updating file: {}", file.path))
                        .unwrap();

                    let absolute_path = format!("{}\\{}", path, file.path);

                    std::fs::remove_file(&absolute_path).unwrap();

                    // Write new resource file
                    let mut file = std::fs::File::create(&absolute_path).unwrap();
                    file.write_all(&res.bytes().await.unwrap()).unwrap();
                }
            }

            status_tx.send("Idle".to_owned()).unwrap();
        });
    }

    pub fn install_patches(
        &self,
        progress_tx: Sender<f64>,
        status_tx: Sender<String>,
        tx: Sender<u32>,
        ctx: egui::Context,
    ) {
        let files = self.cfg.files.clone();
        let path = self.cfg.path.clone();
        let list = self.cfg.realmlist.clone();

        tokio::spawn(async move {
            tx.send(1).unwrap();
            ctx.request_repaint();
            status_tx.send("Patching..".to_owned()).unwrap();

            // overwrite the realmlist since i cba to check if it's outdated
            let dir = fs::read_dir(path.clone())
                .unwrap()
                .filter(|entry| entry.as_ref().unwrap().file_type().unwrap().is_dir());

            for entry in dir {
                let mut e = entry.unwrap().path();

                let mut data_path = e.clone();
                data_path.push("Data");

                // find any folder in this directory which will be as example: enUS/enGB/deDE
                let lang_dir = fs::read_dir(data_path.clone())
                    .unwrap()
                    .filter(|entry| entry.as_ref().unwrap().file_type().unwrap().is_dir())
                    .next()
                    .unwrap()
                    .unwrap();

                let mut realm_path: PathBuf = PathBuf::new();

                // find realmlist.wtf since manually resolving it does not work for some reason and i cba to figure out why
                for entry in fs::read_dir(lang_dir.path()).unwrap() {
                    let e = entry.unwrap();

                    if e.path().to_str().unwrap().contains("realmlist") {
                        realm_path = e.path();
                    }
                }

                let mut file = OpenOptions::new()
                    .write(true)
                    .open(realm_path.clone())
                    .unwrap();

                file.write_all(list.as_bytes()).unwrap();

                for file in &files {
                    status_tx
                        .send(format!("Patching file: {}", file.name))
                        .unwrap();
                    let mut res = reqwest::get(&file.url).await.unwrap();

                    let mut downloaded = 0;
                    let total_size = res.content_length().unwrap_or(0);

                    let absolute_path = &e.join(file.path.clone()).join(file.name.clone());
                    let mut patch_file = std::fs::File::create(&absolute_path).unwrap();

                    while let Some(chunk) = res.chunk().await.unwrap() {
                        downloaded += chunk.len();
                        patch_file.write(&chunk).unwrap();

                        let progress = downloaded as f64 / total_size as f64;

                        progress_tx.send(progress).unwrap();
                    }
                }
            }

            status_tx.send("Idle".to_owned()).unwrap();
        });
    }
}
