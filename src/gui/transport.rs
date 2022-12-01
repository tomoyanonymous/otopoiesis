use crate::data;
use crate::gui;

use nannou::time::DurationF64;
use nannou_egui::egui;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

struct Toggle {
    transport: Arc<data::Transport>,
}
impl Toggle {
    fn new(t: Arc<data::Transport>) -> Self {
        Self {
            transport: t.clone(),
        }
    }
}
impl egui::Widget for Toggle {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let playing = &self.transport.is_playing;
        if playing.load(Ordering::Relaxed) {
            let res = ui.button("pause");
            if res.clicked() {
                playing.store(true, Ordering::Relaxed);
            };
            res
        } else {
            let res = ui.button("play");
            if res.clicked() {
                playing.store(false, Ordering::Relaxed);
            };
            res
        }
    }
}

pub struct Model {
    pub param: Arc<data::Transport>,
    pub sample_rate: u64,
    playbutton: Toggle,
    // pub play_button: egui::Texture,
}
impl Model {
    pub fn new(param: Arc<data::Transport>, sample_rate: u64) -> Self {
        // egui::paint::
        Self {
            param: param.clone(),
            sample_rate,
            playbutton: Toggle::new(param.clone()),
            // play_button,
        }
    }
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
        ui.horizontal(|mut ui| {
            
            let time = std::time::Duration::from_secs_f64(self.get_time());
            if ui.button("|<<").clicked() {
                self.param.time.store(0, Ordering::Relaxed);
            }
            if ui.button("[  ]").clicked() {
                //stop
            }
            ui.add(self.playbutton);

            ui.label(format!(
                "{:02} : {:02} : {:06}",
                time.mins().floor() as u64,
                time.as_secs(),
                time.subsec_micros()
            ));
        })
        .response
    }
}
