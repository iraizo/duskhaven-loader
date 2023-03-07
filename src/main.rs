#![feature(async_fn_in_trait)]

pub mod config;
pub mod installer;
pub mod ui;
pub mod updater;

use egui::{Color32, Style};
use tokio::runtime::Runtime;
use ui::Ui;
use updater::Updater;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1920.0, 1080.0)),
        centered: true,
        ..Default::default()
    };

    let config = config::parse_config();

    let rt = Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    eframe::run_native(
        "Duskhaven Launcher",
        options,
        Box::new(|cc| Box::new(Ui::new(cc, config))),
    )
    .unwrap();
}
