use crate::data;
use crate::gui;
use crate::utils::AtomicRange;

use crate::action;
use egui;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};



pub struct Model {
    pub param: data::Track,
    app: Arc<Mutex<data::AppModel>>,
    regions: Vec<gui::region::Model>,
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
        }
    }
}

impl egui::Widget for &mut Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let offset_x = ui.clip_rect().left();
        let response = ui
            .with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                let track_len;
                let x_rightmost;

                {
                    let rect = ui.min_rect();
                    for region in self.regions.iter_mut() {
                        let x = region.params.range.start() as f32
                            / gui::SAMPLES_PER_PIXEL_DEFAULT as f32;
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                        ui.put(rect.translate([x, 0.0].into()), region);
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
        let text = response.hover_pos().map_or("none".to_string(), |p| {
            format!("{:?}/offset:{}", p, offset_x).to_string()
        });
        response.on_hover_text_at_pointer(text)
    }
}
