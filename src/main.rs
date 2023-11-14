use crate::app;
use otopoiesis::*;
extern crate eframe;
extern crate egui;

#[cfg(not(target_arch = "wasm32"))]
use crate::cli::{self, Parser};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200., 900.)),
        ..Default::default()
    };
    let arg: crate::data::LaunchArg = cli::Args::parse().into();
    eframe::run_native(
        "otopoiesis",
        native_options,
        Box::new(|cc| Box::new(app::Model::new(cc, Some(arg)))),
    )
    .ok();
}

///binary crate for web does nothing.
#[cfg(target_arch = "wasm32")]
fn main() {}
