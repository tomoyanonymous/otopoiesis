//! Otopoiesis is a constructive audio editing environment.
//!

extern crate eframe;
extern crate egui;
extern crate serde_json;

pub mod action;
pub mod app;
pub mod audio;
pub mod data;
pub mod gui;
pub mod parameter;
pub mod utils;

#[cfg(feature = "native")]
pub mod cli;

#[cfg(feature = "web")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(feature = "web")]
extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[cfg(feature = "web")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(feature = "web")]
#[wasm_bindgen]
pub struct WebHandle {
    #[allow(dead_code)]
    handle: eframe::web::AppRunnerRef,
}

/// Call this once from the HTML.
#[cfg(feature = "web")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<WebHandle, eframe::wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        canvas_id,
        web_options,
        Box::new(|cc| Box::new(app::Model::new(cc, None))),
    )
    .await
    .map(|handle| WebHandle { handle })
}
