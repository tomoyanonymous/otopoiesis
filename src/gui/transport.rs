use crate::atomic::U64;
use crate::data;
use crate::{atomic::SimpleAtomic, audio::renderer::PlayState};
use std::sync::{Arc, mpsc};
struct Toggle {
    pub controller: mpsc::Sender<data::PlayOp>,
    pub playstate: Arc<PlayState>,
}

impl Toggle {
    fn new(controller: mpsc::Sender<data::PlayOp>, playstate: Arc<PlayState>) -> Self {
        Self {
            controller,
            playstate,
        }
    }
}
impl egui::Widget for &mut Toggle {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let res = if *self.playstate == PlayState::Playing {
            ui.button("⏸")
        } else {
            ui.button("▶")
        };
        if res.clicked() {
            self.controller.send(data::PlayOp::Toggle).unwrap();
        };
        res
    }
}

pub struct Model {
    pub sample_rate: u64,
    pub current_time: U64,
    playbutton: Toggle,
    // pub play_button: egui::Texture,
}
impl Model {
    pub fn new(
        controller: mpsc::Sender<data::PlayOp>,
        sample_rate: u64,
        current_time: U64,
    ) -> Self {
        let playbutton = Toggle::new(controller, Arc::new(PlayState::Stopped));
        Self {
            sample_rate,
            current_time,
            playbutton,
        }
    }
    fn get_time_in_sample(&self) -> u64 {
        self.current_time.load()
    }
    fn get_time(&self) -> f64 {
        self.get_time_in_sample() as f64 / self.sample_rate as f64
    }
    // fn is_playing(&self) -> bool {
    //     self.param.is_playing()
    // }
}

impl egui::Widget for &mut Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        for (_text_style, font_id) in ui.style_mut().text_styles.iter_mut() {
            font_id.size = 24.0 // whatever size you want here
        }
        ui.horizontal(|ui| {
            let time = std::time::Duration::from_secs_f64(self.get_time());

            if ui.button("⏮").clicked() {
                self.playbutton
                    .controller
                    .send(data::PlayOp::JumpTo(0))
                    .unwrap();

                self.current_time.store(0);
            }
            if ui.button("⏹").clicked() {
                self.playbutton.controller.send(data::PlayOp::Halt).unwrap();
            }
            ui.add(&mut self.playbutton);
            let min = time.div_f64(60.0).as_secs();
            let secs = time.as_secs() % 60;
            ui.label(format!(
                "{:02} : {:02} : {:06}",
                min,
                secs,
                time.subsec_micros()
            ));
        })
        .response
    }
}
