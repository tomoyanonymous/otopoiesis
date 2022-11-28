use crate::data;
use crate::gui;
use crate::parameter::Parameter;
use crate::utils::AtomicRange;
use nannou_egui::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Model {
    pub param: Arc<data::Project>,
}

impl Model {
    pub fn new(param:Arc<data::Project>)->Self{
        Self { param:Arc::clone(&param) }
    }
    fn get_samplerate(&self) -> u64 {
        self.param.sample_rate.load(Ordering::Relaxed)
    }

}

impl egui::Widget for Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let main = egui::ScrollArea::horizontal().show(ui, |ui| {


            let res = ui
                .vertical(|ui| {

                    if let Ok(tracks) = self.param.tracks.try_lock() {
                        for (_i, track) in tracks.iter().enumerate() {
                            ui.add(gui::track::Model {
                                param: data::Track(track.0.clone()),
                            });
                        }
                    }
                })
                .response;

            // painter.debug_text(
            //     rect.center(),
            //     egui::Align2::LEFT_BOTTOM,
            //     egui::Color32::GRAY,
            //     format!("time x:{}", x),
            // );
            // painter.debug_rect(rect, egui::Color32::GRAY, "timeline");
            res
        });
        main
    }
}
