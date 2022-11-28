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
            let x =
                self.time * self.params.sample_rate as f64 / gui::SAMPLES_PER_PIXEL_DEFAULT as f64;

            let res = ui
                .vertical(|ui| {
                    let label = if self.played.into_inner() {
                        "played"
                    } else {
                        "paused"
                    };
                    ui.label(format!("{}", label));

                    if let Ok(tracks) = self.params.tracks.try_lock() {
                        for (_i, track) in tracks.iter().enumerate() {
                            ui.add(gui::track::Model {
                                param: data::Track(track.0.clone()),
                            })
                            .interact(egui::Sense::click_and_drag());
                        }
                    }
                })
                .response;
            let stroke = egui::Stroke::new(3.0, egui::Color32::GRAY);
            let mut painter = ui.painter_at(ui.clip_rect());
            let rect = painter.clip_rect();
            painter.line_segment(
                [
                    egui::pos2(x as f32, rect.top()),
                    egui::pos2(x as f32, rect.bottom()),
                ],
                stroke,
            );
            painter.debug_text(
                rect.center(),
                egui::Align2::LEFT_BOTTOM,
                egui::Color32::GRAY,
                format!("time x:{}", x),
            );
            painter.debug_rect(rect, egui::Color32::GRAY, "timeline");
            res
        });
        main
    }
}
