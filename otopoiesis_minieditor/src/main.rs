extern crate eframe;
extern crate egui;
mod app;

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200., 900.)),
        ..Default::default()
    };

    eframe::run_native(
        "otopoiesis_minieditor",
        native_options,
        Box::new(|cc| Box::new(app::Model::new(cc))),
    )
    .ok();
}