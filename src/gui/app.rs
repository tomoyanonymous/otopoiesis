use crate::data;
use crate::gui;
use nannou_egui::egui;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard};
use undo;
pub struct Model {
    pub param: Arc<Mutex<data::AppModel>>,
}

impl Model {
    fn get_model_mut(&self) -> MutexGuard<data::AppModel> {
        self.param.lock().unwrap()
    }
    fn get_timeline(&self) -> Arc<data::Project> {
        self.get_model_mut().project.clone()
    }
    fn get_transport(&self) -> Arc<data::Transport> {
        self.get_model_mut().transport.clone()
    }
    pub fn show_ui(&mut self, ctx: &egui::CtxRef) {
        egui::panel::TopBottomPanel::top("header").show(&ctx, |ui| {
            ui.label("otopoiesis");
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.add(gui::timeline::Model {
                param: self.get_timeline(),
                app: Arc::clone(&self.param),
            });
        });
        egui::panel::TopBottomPanel::bottom("footer").show(&ctx, |ui| {
            ui.add(gui::transport::Model {
                param: self.get_transport(),
                sample_rate: self.get_timeline().sample_rate.load(Ordering::Relaxed),
            })
        });
    }
}
