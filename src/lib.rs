//! Otopoiesis is a constructive audio editing environment.
//!

extern crate eframe;
extern crate egui;
extern crate serde_json;

pub mod app;
pub mod action;
pub mod audio;
pub mod data;
pub mod gui;
pub mod parameter;
pub mod utils;


#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WebHandle {
    handle: eframe::web::AppRunnerRef,
}


/// Call this once from the HTML.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<WebHandle, eframe::wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        canvas_id,
        web_options,
        Box::new(|cc| Box::new(appModel::new(cc))),
    )
    .await
    .map(|handle| WebHandle { handle })
}



