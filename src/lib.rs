//! Otopoiesis is a constructive audio editing environment.
//!
#![feature(box_patterns)]
#![feature(iterator_try_collect)]

extern crate eframe;
extern crate egui;
extern crate serde_json;

pub mod action;
pub mod app;
pub mod audio;
pub mod data;
pub mod gui;
pub mod parameter;
pub mod script;
pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
pub mod cli;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;

#[cfg(target_arch = "wasm32")]
extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Clone)]
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WebHandle {
    #[allow(dead_code)]
    runner: eframe::WebRunner,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    /// Installs a panic hook, then returns.
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        // Redirect [`log`] message to `console.log` and friends:
        if cfg(debug_assertions) {
            eframe::web::WebLogger::init(log::LevelFilter::Debug).ok();
        } else {
            eframe::web::WebLogger::init(log::LevelFilter::Trace).ok();
        }
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    /// Call this once from JavaScript to start your app.
    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
        self.runner
            .start(
                canvas_id,
                eframe::WebOptions::default(),
                Box::new(|cc| Box::new(app::Model::new(cc, None))),
            )
            .await
    }
}
