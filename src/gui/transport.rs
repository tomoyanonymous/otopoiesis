use crate::data;
use crate::gui;

use nannou::time::DurationF64;
use nannou_egui::egui;
use std::sync::atomic::{ Ordering};
use std::sync::Arc;

pub struct Model {
    pub param: Arc<data::Transport>,
    pub sample_rate: u64,
}
impl Model {
    fn get_time_in_sample(&self) -> u64 {
        self.param.time.load(Ordering::Relaxed)
    }
    fn get_time(&self) -> f64 {
        self.get_time_in_sample() as f64 / self.sample_rate as f64
    }
    fn is_playing(&self) -> bool {
        self.param.is_playing.load(Ordering::Relaxed)
    }
}

impl egui::Widget for Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let x = self.get_time_in_sample() as f64 / gui::SAMPLES_PER_PIXEL_DEFAULT as f64;
            let time = std::time::Duration::from_secs_f64(self.get_time());
            let label = if self.is_playing() {
                "played"
            } else {
                "paused"
            };
            ui.label(format!(
                "{:02} : {:02} : {:06}",
                time.mins().floor() as u64,
                time.as_secs(),
                time.subsec_micros()
            ));
        })
        .response

        // ui.label(format!("{}", label));
        // let stroke = egui::Stroke::new(3.0, egui::Color32::GRAY);
        // let mut painter = ui.painter_at(ui.clip_rect());
        // let rect = painter.clip_rect();
        // painter.line_segment(
        //     [
        //         egui::pos2(x as f32, rect.top()),
        //         egui::pos2(x as f32, rect.bottom()),
        //     ],
        //     stroke,
        // );
    }
}
