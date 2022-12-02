use crate::data;
use crate::gui;
use crate::parameter::{FloatParameter, Parameter};
use crate::utils::AtomicRange;

use crate::action;
use egui;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use undo::{Action, Record};

pub struct Model {
    pub param: data::Track,
    app: Arc<Mutex<data::AppModel>>,
    regions: Vec<gui::region::Model>,
    scroll_x: f32,
}

impl Model {
    pub fn new(param: data::Track, app: Arc<Mutex<data::AppModel>>) -> Self {
        let track = param.0.lock().unwrap();
        let regions = track
            .iter()
            .map(|region| gui::region::Model::new(region.clone(), region.label.clone()))
            .collect::<Vec<_>>();

        Self {
            param: param.clone(),
            app: app.clone(),
            regions,
            scroll_x: 0.0,
        }
    }
}
impl Model {
    pub fn set_offset_x(&mut self, newx: f32) {
        self.scroll_x = newx
    }
}
impl egui::Widget for &mut Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui
            .with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                let track_len;
                let x_rightmost;

                {
                    let rect = ui.min_rect();
                    for region in self.regions.iter_mut() {
                        let _response = ui.put(rect, region);
                    }
                    let track = self.param.0.lock().unwrap();

                    track_len = track.len() + 1;
                    x_rightmost = track
                        .iter()
                        .fold(0u64, |acc, region| acc.max(region.range.end()));
                } //first lock drops here
                if ui.button("+").clicked() {
                    let label = format!("region{}", track_len).to_string();
                    let region_param = Arc::new(data::Region {
                        range: AtomicRange::new(x_rightmost, x_rightmost + 49000),
                        max_size: AtomicU64::from(60000),
                        generator: Arc::new(data::Generator::Oscillator(Arc::new(
                            data::OscillatorParam::default(),
                        ))),
                        filters: vec![],
                        label,
                    });
                    let mut app = self.app.lock().unwrap();
                    let _res = action::add_region(&mut app, self.param.0.clone(), region_param);
                }
            })
            .response;
        let text = response
            .hover_pos()
            .map_or("none".to_string(), |p| format!("{:?}", p).to_string());
        response.on_hover_text_at_pointer(text)
    }
}
