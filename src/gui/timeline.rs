use crate::action;
use crate::data;
use crate::gui;
use crate::utils::atomic;
use std::sync::{Arc, Mutex};

pub struct Model {
    pub app: Arc<Mutex<data::AppModel>>,
    time: Arc<atomic::U64>,
    track: Vec<gui::track::Model>,
}

fn param_to_track(
    track_p: &[data::Track],
    app: Arc<Mutex<data::AppModel>>,
) -> Vec<gui::track::Model> {
    track_p
        .iter()
        .enumerate()
        .map(|(i, t)| gui::track::Model::new(t.clone(), app.clone(), i))
        .collect::<Vec<_>>()
}

impl Model {
    pub fn new(
        param: data::Project,
        app: Arc<Mutex<data::AppModel>>,
        time: Arc<atomic::U64>,
    ) -> Self {
        let track = param_to_track(&param.tracks, app.clone());
        Self {
            app: Arc::clone(&app),
            time,
            track,
        }
    }

    // fn get_samplerate(&self) -> u64 {
    //     self.param.sample_rate.load(Ordering::Relaxed)
    // }
    fn get_current_time_in_sample(&self) -> u64 {
        self.time.load()
    }
    fn draw_frame(&mut self, painter: &egui::Painter, style: &egui::Style) {
        let rect = painter.clip_rect();
        painter.rect_stroke(
            rect,
            5.0,
            egui::Stroke::new(2.0, style.visuals.extreme_bg_color), //tekitou
        );
    }
    fn draw_current_time(&mut self, painter: &egui::Painter, style: &egui::Style) {
        let stroke = style.visuals.window_stroke();

        let rect = painter.clip_rect();

        let x = self.get_current_time_in_sample() as f32 / gui::SAMPLES_PER_PIXEL_DEFAULT as f32
            + rect.left();
        painter.line_segment(
            [
                [x as f32, rect.top()].into(),
                [x as f32, rect.bottom()].into(),
            ],
            stroke,
        );
    }
    fn add_track(&mut self) {
        let new_track = if let Ok(mut app) = self.app.lock() {
            let _res = action::add_track(&mut app, data::Track::new());
            param_to_track(&app.project.tracks, self.app.clone())
        } else {
            vec![]
        };
        self.track = new_track;
    }
    pub fn sync_state(&mut self, track_p: &[data::Track]) {
        self.track = param_to_track(track_p, self.app.clone());
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
            let add_track_button = ui.button("add track");
            if add_track_button.clicked() {
                self.add_track();
            }

            let painter = ui.painter_at(ui.clip_rect());
            self.draw_frame(&painter, ui.style());
            self.draw_current_time(&painter, ui.style());

            res
        });

        main.inner
    }
}
