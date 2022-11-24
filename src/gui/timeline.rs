use crate::data;
use crate::gui;
use crate::parameter::Parameter;
use crate::utils::AtomicRange;
use crate::*;
use nannou_egui::egui;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct Model {
    pub time: f64,
    pub played: AtomicBool,
    pub params: Arc<data::Project>,
}

impl egui::Widget for Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let main = egui::ScrollArea::horizontal().show(ui, |ui| {
            let x = self.time * 100.;//ratio is tekitou

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
            let stroke = egui::Stroke::new(3.0, egui::Color32::GREEN);
            let mut painter =  ui.painter_at(ui.clip_rect());
            let rect = painter.clip_rect();
            painter.line_segment(
                [egui::pos2(x as f32, rect.top()), egui::pos2(x as f32, rect.bottom())],
                stroke,
            );
            painter.debug_text(rect.center(), egui::Align2::LEFT_BOTTOM, egui::Color32::GREEN, format!("time:{}",x));
            painter.debug_rect(rect, egui::Color32::GREEN, "timeline");
            res
        });
        main
    }
}
