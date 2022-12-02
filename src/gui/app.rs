use crate::data;
use crate::gui;
use egui;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard};
use undo;
pub struct Model {
    pub param: Arc<Mutex<data::AppModel>>,
    timeline: gui::timeline::Model,
}

impl Model {
    pub fn new(param: Arc<Mutex<data::AppModel>>) -> Self {
        let p = param.clone();
        let t = param.lock().unwrap().project.clone();

        Self {
            param: p.clone(),
            timeline: gui::timeline::Model::new(t, p.clone()),
        }
    }

    fn get_model_mut(&self) -> MutexGuard<data::AppModel> {
        self.param.lock().unwrap()
    }
    fn get_timeline(&self) -> Arc<data::Project> {
        self.get_model_mut().project.clone()
    }
    fn get_transport(&self) -> Arc<data::Transport> {
        self.get_model_mut().transport.clone()
    }
    pub fn show_ui(&mut self, ctx: &egui::Context) {
        egui::panel::TopBottomPanel::top("header").show(&ctx, |ui| {
            ui.label("otopoiesis");
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.add(&mut self.timeline);
        });
        egui::panel::TopBottomPanel::bottom("footer").show(&ctx, |ui| {
            ui.add(gui::transport::Model::new(
                self.get_transport(),
                self.get_timeline().sample_rate.load(Ordering::Relaxed),
            ))
        });
    }
}
