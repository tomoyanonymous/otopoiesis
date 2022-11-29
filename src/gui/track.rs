use crate::data;
use crate::gui::region;
use crate::parameter::{FloatParameter, Parameter};
use crate::utils::AtomicRange;

use crate::action;
use nannou_egui::egui;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use undo::{Action, Record};

pub struct Model {
    pub param: data::Track,
    pub app: Arc<Mutex<data::AppModel>>,
}
impl egui::Widget for Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            let track_len;
            {
                let track = self.param.0.lock().unwrap();
                //regions maybe overlap to each other, so we need to split layer
                track_len = track.len() + 1;

                for (_i, region) in track.iter().enumerate() {
                    let model = region::Model::new(Arc::clone(region), region.label.clone());

                    let _response = ui.add(model);
                }
            } //first lock drops here
            let label = format!("region{}", track_len).to_string();

            if ui.button("add_region").clicked() {
                let osc_param = Arc::new(data::OscillatorParam {
                    amp: FloatParameter::new(1.0, 0.0..=1.0, "amp"),
                    freq: FloatParameter::new(440.0, 20.0..=20000.0, "freq"),
                    phase: FloatParameter::new(0.0, 0.0..=6.3, "phase"),
                });
                let region_param = Arc::new(data::Region {
                    range: AtomicRange::new(1000, 50000),
                    max_size: AtomicU64::from(60000),
                    generator: Arc::new(data::Generator::Oscillator(Arc::clone(&osc_param))),
                    filters: vec![],
                    label,
                });
                let mut app = self.app.lock().unwrap();

                let _res = action::add_region(&mut app, self.param.0.clone(), region_param);
            }

            // let _res = app.history.apply((),action).unwrap();
            // track.push(region_param);
        })
        .response
    }
}
