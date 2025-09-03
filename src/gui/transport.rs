use crate::atomic;
use crate::data;
use crate::{atomic::SimpleAtomic, audio::renderer::PlayState};
use std::sync::{Arc, mpsc};
struct Toggle<'a> {
    pub playstate: &'a mut PlayState,
}

impl<'a> Toggle<'a> {
    fn new(playstate: &'a mut PlayState) -> Self {
        Self { playstate }
    }
}
impl<'a> egui::Widget for Toggle<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let res = if *self.playstate == PlayState::Playing {
            ui.button("⏸")
        } else {
            ui.button("▶")
        };
        if res.clicked() {
            *self.playstate = self.playstate.update_state_by_op(data::PlayOp::Toggle);
        };
        res
    }
}

pub struct Model<'a> {
    pub sample_rate: u64,
    pub current_time: Arc<atomic::U64>,
    playbutton: Toggle<'a>,
    // pub play_button: egui::Texture,
}
impl<'a> Model<'a> {
    pub fn new(
        playstate: &'a mut PlayState,
        sample_rate: u64,
        current_time: Arc<atomic::U64>,
    ) -> Self {
        let playbutton = Toggle::new(playstate);
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

impl<'a> egui::Widget for Model<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        for (_text_style, font_id) in ui.style_mut().text_styles.iter_mut() {
            font_id.size = 24.0 // whatever size you want here
        }
        ui.horizontal(|ui| {
            let time = std::time::Duration::from_secs_f64(self.get_time());

            if ui.button("⏮").clicked() {
                self.current_time.store(0);
            }
            if ui.button("⏹").clicked() {
                *self.playbutton.playstate = self
                    .playbutton
                    .playstate
                    .update_state_by_op(data::PlayOp::Halt);
            }
            ui.add(self.playbutton);   
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
