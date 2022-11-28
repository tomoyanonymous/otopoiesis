use crate::data;
use crate::gui;
use nannou_egui::egui;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct Model {
    pub param: Arc<data::AppModel>,
}

impl Model {
    fn get_timeline(&self) -> &Arc<data::Project> {
        &self.param.project
    }
    fn get_transport(&self) -> &Arc<data::Transport> {
        &self.param.transport
    }
    pub fn show_ui(&mut self, ctx: &egui::CtxRef) {
        egui::panel::TopBottomPanel::top("header").show(&ctx, |ui| {
            ui.label("otopoiesis");
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.add(gui::timeline::Model {
                param: Arc::clone(self.get_timeline()),
            });
        });
        egui::panel::TopBottomPanel::bottom("footer").show(&ctx, |ui| {
            ui.add(gui::transport::Model {
                param: Arc::clone(self.get_transport()),
                sample_rate: self.get_timeline().sample_rate.load(Ordering::Relaxed),
            })
        });
    }
}
