use crate::data;
use crate::gui;
use crate::parameter::Parameter;
use crate::utils::AtomicRange;
use egui;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub struct Model {
    pub param: Arc<data::Project>,
    pub app: Arc<Mutex<data::AppModel>>,
    time: Arc<AtomicU64>,
    track: Vec<gui::track::Model>,
}

impl Model {
    pub fn new(param: Arc<data::Project>, app: Arc<Mutex<data::AppModel>>) -> Self {
        let time = app.lock().unwrap().transport.time.clone();
        let track = param
            .tracks
            .lock()
            .unwrap()
            .iter()
            .map(|t| gui::track::Model::new(data::Track(t.0.clone()), app.clone()))
            .collect::<Vec<_>>();
        Self {
            param: Arc::clone(&param),
            app: Arc::clone(&app),
            time,
            track,
        }
    }
    fn get_samplerate(&self) -> u64 {
        self.param.sample_rate.load(Ordering::Relaxed)
    }
    fn get_current_time_in_sample(&self) -> u64 {
        self.time.load(Ordering::Relaxed)
    }
}

impl egui::Widget for &mut Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let main = egui::ScrollArea::horizontal().show(ui, |ui| {
            let res = ui
                .vertical(|ui| {
                    for track in self.track.iter_mut() {
                        ui.add(track);
                    }
                })
                .response;

            let stroke = egui::Stroke::new(3.0, egui::Color32::GRAY);
            let painter = ui.painter_at(ui.clip_rect());
            let rect = painter.clip_rect();

            let x = self.get_current_time_in_sample() as f32
                / gui::SAMPLES_PER_PIXEL_DEFAULT as f32
                + rect.left();
            painter.line_segment(
                [
                    [x as f32, rect.top()].into(),
                    [x as f32, rect.bottom()].into(),
                ],
                stroke,
            );
            res
        });

        main.inner
    }
}
