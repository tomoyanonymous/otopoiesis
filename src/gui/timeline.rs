use crate::action;
use crate::action::Action;
use crate::action::AddTrack;
use crate::data;
use crate::gui;
use crate::utils::atomic::{self, SimpleAtomic};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub struct State {
    track: Vec<gui::track::State>,
    now: Arc<atomic::U64>,
    sample_rate: u64,
}
impl State {
    pub fn new(track_p: &[data::Track], now: Arc<atomic::U64>, sample_rate: u64) -> Self {
        Self {
            track: param_to_track(track_p),
            now,
            sample_rate,
        }
    }
    pub fn sync_state(&mut self, track_p: &[data::Track]) {
        self.track = param_to_track(track_p);
    }
}

pub struct Model<'a> {
    // pub action_sender: mpsc::Sender<Action>,
    app: &'a mut data::AppModel,
    state: &'a mut State,
}

fn param_to_track(track_p: &[data::Track]) -> Vec<gui::track::State> {
    track_p
        .iter()
        .enumerate()
        .map(|(_i, t)| gui::track::State::new(t, 5))
        .collect::<Vec<_>>()
}

impl<'a> Model<'a> {
    pub fn new(app: &'a mut data::AppModel, state: &'a mut State) -> Self {
        Self { app, state }
    }

    // fn get_samplerate(&self) -> u64 {
    //     self.param.sample_rate.load(Ordering::Relaxed)
    // }
    fn get_current_time_in_sample(&self) -> u64 {
        self.state.now.load()
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
        let sr = self.state.sample_rate;
        let x = (self.get_current_time_in_sample() as f64 * gui::PIXELS_PER_SEC_DEFAULT as f64
            / sr as f64) as f32
            + rect.left();
        painter.line_segment([[x, rect.top()].into(), [x, rect.bottom()].into()], stroke);
    }
    fn add_track(&self) {
        // if let Ok(mut app) = self.app.lock() {
        self.app
            .action_tx
            .send(Action::from(AddTrack::new(data::Track::new())));

        // self.state.track = param_to_track(&app.project.tracks);
    }
}

impl<'a> egui::Widget for Model<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let main = egui::ScrollArea::horizontal().show(ui, |ui| {
            let res = ui
                .vertical(|ui| {
                    for (i, state) in self.state.track.iter_mut().enumerate() {
                        ui.add(gui::track::Model::new(
                            i,
                            self.app.action_tx.clone(),
                            self.app.get_track_for_id_mut(i).unwrap(),
                            state,
                        ));
                        ui.add_space(30.0);
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
