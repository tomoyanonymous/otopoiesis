use crate::data;
use crate::gui;
use crate::parameter::Parameter;
use crate::utils::AtomicRange;
use crate::*;
use nannou_egui::egui;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct Model {
    pub time: u64,
    pub played: AtomicBool,
    pub params: Arc<data::Project>,
}

impl egui::Widget for Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let main = egui::ScrollArea::horizontal().show(ui, |ui| {
            let res = ui
                .vertical(|ui| {
                    let label = if self.played.into_inner() {
                        "played"
                    } else {
                        "paused"
                    };
                    ui.label(format!("{}", label));
                    for (_i, track) in self.params.tracks.iter().enumerate() {
                        ui.add(gui::track::Model {
                            param: Arc::clone(&track),
                        })
                        .interact(egui::Sense::click_and_drag());
                    }
                })
                .response;
            let x = self.time as f32 / 100.;
            let stroke = egui::Stroke::new(3.0, egui::Color32::GREEN);
            let painter = ui.painter();
            let rect = painter.clip_rect();
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                stroke,
            );
            res
        });
        main
    }
}
