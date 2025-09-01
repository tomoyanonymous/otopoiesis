use egui::StrokeKind;

use crate::action::{Action, AddTrack};
use crate::atomic::{self, SimpleAtomic};
use crate::data;
use crate::gui;
use crate::script::Expr;
use std::sync::Arc;

pub struct Model<'a> {
    app: &'a mut data::AppModel,
}

impl<'a> Model<'a> {
    pub fn new(app: &'a mut data::AppModel) -> Self {
        Self { app }
    }

    // fn get_samplerate(&self) -> u64 {
    //     self.param.sample_rate.load(Ordering::Relaxed)
    // }
    fn get_current_time_in_sample(&self) -> u64 {
        self.app.project.current_time.load()
    }
    #[allow(dead_code)]
    fn draw_frame(&mut self, painter: &egui::Painter, style: &egui::Style) {
        let rect = painter.clip_rect();
        painter.rect_stroke(
            rect,
            5.0,
            egui::Stroke::new(2.0, style.visuals.extreme_bg_color), //tekitou
            StrokeKind::Inside,
        );
    }
    fn draw_current_time(&mut self, painter: &egui::Painter, style: &egui::Style) {
        let stroke = style.visuals.window_stroke();

        let rect = painter.clip_rect();
        let sr = self.app.project.sample_rate.load();
        let x = (self.get_current_time_in_sample() as f64 * gui::PIXELS_PER_SEC_DEFAULT as f64
            / sr as f64) as f32
            + rect.left();
        painter.line_segment([[x, rect.top()].into(), [x, rect.bottom()].into()], stroke);
    }
    fn add_track(&self) {
        // let _ = self
        //     .app
        //     .action_tx
        //     .send(Action::from(AddTrack::new(Expr::Track(
        //         Expr::Array(vec![]).into(),
        //     ))));
    }
}

impl<'a> egui::Widget for Model<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let main = egui::ScrollArea::horizontal().show(ui, |ui| {
            let res = ui
                .vertical(|ui| {
                    let sender = self.app.action_tx.clone();
                    for (i, track) in self.app.project.tracks.iter_mut().enumerate() {
                        ui.push_id(i, |ui| {
                            ui.add(gui::track::Model::new(i, sender.clone(), track));
                            ui.add_space(30.0);
                        })
                        .inner
                    }
                })
                .response;
            let add_track_button = ui.button("+").on_hover_text("Add new Track");
            if add_track_button.clicked() {
                self.add_track();
            }

            let painter = ui.painter_at(ui.clip_rect());
            self.draw_current_time(&painter, ui.style());

            res
        });

        main.inner
    }
}
