use crate::app;
use otopoiesis::*;
extern crate eframe;
extern crate egui;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::vec2(1200., 900.));
    eframe::run_native(
        "otopoiesis",
        native_options,
        Box::new(|cc| Box::new(app::Model::new(cc))),
    );
}

#[cfg(target_arch = "wasm32")]
///binary crate for web does nothing.
fn main() {}
