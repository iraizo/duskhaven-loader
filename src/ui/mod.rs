use std::{
    process::Command,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

use egui::{menu, Color32, FontId, ProgressBar, Style, TextStyle};
use lazy_static::lazy_static;
use native_dialog::{MessageDialog, MessageType};
use windows::Win32::System::WindowsProgramming::GetUserNameA;

use crate::{config::Configuration, installer::Installer, updater::Updater};

lazy_static! {
    static ref TAB: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    static ref UPDATE_TEXT: Arc<Mutex<String>> =
        Arc::new(Mutex::new(String::from("Check for Updates")));
}

pub struct Ui {
    pub name: String,
    pub cfg: Configuration,
    pub update_text: String,
    pub installing_text: String,
    pub status_text: String,
    pub install_status: bool,
    pub progress: f64,

    tx: Sender<u32>,
    rx: Receiver<u32>,

    tx_install: Sender<u32>,
    rx_install: Receiver<u32>,

    tx_status: Sender<String>,
    rx_status: Receiver<String>,

    tx_progress_install: Sender<f64>,
    rx_progress_install: Receiver<f64>,
}

impl Ui {
    pub fn new(cc: &eframe::CreationContext<'_>, cfg: Configuration) -> Self {
        let mut style = Style::default();
        style.visuals.override_text_color = Some(Color32::WHITE);

        style.text_styles = [
            (
                TextStyle::Body,
                FontId::new(25.0, egui::FontFamily::Proportional),
            ),
            (
                TextStyle::Button,
                FontId::new(25.0, egui::FontFamily::Proportional),
            ),
            (
                TextStyle::Heading,
                FontId::new(25.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();

        cc.egui_ctx.set_style(style);

        // initalize

        unsafe {
            let mut user: [u8; 256] = [0; 256];
            let mut user_len = user.len() as u32;

            GetUserNameA(
                windows::core::PSTR::from_raw(user.as_mut_ptr()),
                &mut user_len,
            );

            let (tx, rx) = std::sync::mpsc::channel();
            let (tx_install, rx_install) = std::sync::mpsc::channel();
            let (tx_status, rx_status) = std::sync::mpsc::channel();
            let (tx_progress_install, rx_progress_install) = std::sync::mpsc::channel();

            tx_status.send("Idle".to_owned()).unwrap();

            return Self {
                name: String::from_utf8_lossy(&user[..user_len as usize]).to_string(),
                cfg,
                update_text: String::from("Check for Updates"),
                installing_text: String::from("Install"),
                status_text: String::from("Idle"),
                tx,
                rx,
                progress: 0.0,
                tx_install,
                rx_install,
                tx_status,
                rx_status,
                tx_progress_install,
                rx_progress_install,
                install_status: false,
            };
        }
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            menu::bar(ui, |ui| {
                if ui.button("About").clicked() {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("About")
                        .set_text("Developed by raizo.\nBeta build.")
                        .show_alert()
                        .unwrap();
                }
                if ui.button("Main").clicked() {
                    *TAB.lock().unwrap() = 0;
                }
                if ui.button("Settings").clicked() {
                    *TAB.lock().unwrap() = 1;
                }
                ui.add_space(50.0);
                if ui.button("Addons").clicked() {
                    *TAB.lock().unwrap() = 0;
                }
            });

            // main

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
            });

            match *TAB.lock().unwrap() {
                0 => {
                    ui.label(format!("Hello there {}, ready to get started?", self.name));
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label("Instructions");
                    ui.add_space(10.0);
                    ui.label("Click Install if you want to have a full install or Update if you want to patch a current wow 3.3.5a installation");
                    ui.add_space(5.0);
                    ui.label("remember to set your wow folder in the settings if you want to patch your current installation");
                }
                1 => {
                    ui.heading("Settings");
                    ui.separator();
                    ui.label("Set your game path here");
                    if ui.button("Select Folder").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.cfg.path = Some(path.display().to_string()).unwrap();
                            self.cfg.write();
                        }
                    }
                    // add space
                    ui.add_space(5.0);
                    ui.label(format!("Current directory: {}", self.cfg.path));
                }
                _ => {
                    todo!()
                }
            }
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
            });

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                if ui.button("Launch").clicked() {
                    match Command::new(self.cfg.path.clone() + "\\Wow.exe").spawn() {
                        Ok(_) => (),
                        Err(e) => {
                            MessageDialog::new()
                                .set_type(MessageType::Error)
                                .set_title("Error")
                                .set_text(&format!("Failed to launch the game: {}", e))
                                .show_alert()
                                .unwrap();
                        }
                    }
                }

                if let Ok(status) = self.rx.try_recv() {
                    if status == 1 {
                        self.update_text = String::from("Updating..");
                    } else {
                        self.update_text = String::from("Check for Updates");
                    }
                }

                if ui.button(&self.update_text).clicked() {
                    // Its kinda shit to create a new instance everytime but its not resource intensive anyways
                    // but i kinda have to since i cant implement a lifetime without the egui trait crying around.
                    Updater::new(self.cfg.clone()).check(
                        self.tx_status.clone(),
                        self.tx.clone(),
                        ctx.clone(),
                    );
                }

                if let Ok(install_status) = self.rx_install.try_recv() {
                    self.install_status = install_status != 0;
                    if install_status == 1 {
                        self.installing_text = String::from("Installing..");
                    } else {
                        self.installing_text = String::from("Install");
                    }
                }

                if ui.button("Install").clicked() {
                    Updater::new(self.cfg.clone()).install_patches(
                        self.tx_progress_install.clone(),
                        self.tx_status.clone(),
                        self.tx.clone(),
                        ctx.clone(),
                    );
                    /*
                    Installer::new(self.cfg.clone()).clean_install(
                        self.tx_status.clone(),
                        self.tx_install.clone(),
                        self.tx.clone(),
                        self.tx_progress_install.clone(),
                        ctx.clone(),
                    ); */
                }

                if let Ok(status) = self.rx_status.try_recv() {
                    self.status_text = status;
                }

                // add x spacing
                ui.add_space(200.0);
                ui.label(format!("Status: {}", self.status_text));
                ui.add_space(50.0);

                if let Ok(progress) = self.rx_progress_install.try_recv() {
                    self.progress = progress;
                }

                let bar = ProgressBar::new(self.progress as f32).animate(true);

                if self.progress != 0.0 {
                    ui.label(format!("{:.2}%", self.progress * 100.0));
                    ui.add(bar);
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
            });
        });
    }
}
