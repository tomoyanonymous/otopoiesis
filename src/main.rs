use crate::app;
use otopoiesis::*;
extern crate eframe;
extern crate egui;

#[cfg(not(feature = "web"))]
use crate::cli::{self, Parser};

#[cfg(not(feature = "web"))]
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

#[cfg(feature = "web")]

///binary crate for web does nothing.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "main_canvas", // hardcode it
                web_options,
                Box::new(|cc| Box::new(app::Model::new(cc, None))),
            )
            .await
            .expect("failed to start eframe");
    });
}
