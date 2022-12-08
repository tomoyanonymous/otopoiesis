use crate::action;
use crate::data;
use crate::gui;
use crate::utils::{atomic, AtomicRange};

use egui;

use std::sync::{Arc, Mutex};

pub struct Model {
    pub param: data::Track,
    app: Arc<Mutex<data::AppModel>>,
    regions: Vec<gui::region::Model>,
}
fn get_region_from_param(track: &data::Track) -> Vec<gui::region::Model> {
    track
        .0
        .lock()
        .unwrap()
        .iter()
        .map(|region| gui::region::Model::new(region.clone(), region.label.clone()))
        .collect::<Vec<_>>()
}

impl Model {
    pub fn new(param: data::Track, app: Arc<Mutex<data::AppModel>>) -> Self {
        let regions = get_region_from_param(&param);
        Self {
            param: param.clone(),
            app: app.clone(),
            regions,
        }
    }
    fn add_region(&mut self) {
        let x_rightmost = self
            .regions
            .iter()
            .fold(0u64, |acc, region| acc.max(region.params.range.end()));
        let label = format!("region{}", self.regions.len() + 1).to_string();
        let region_param = Arc::new(data::Region {
            range: AtomicRange::new(x_rightmost, x_rightmost + 49000),
            max_size: atomic::U64::from(60000),
            generator: Arc::new(data::Generator::Oscillator(Arc::new(
                data::OscillatorParam::default(),
            ))),
            label,
        });
        let faderegion_p = data::Region::with_fade(region_param);
        {
            let mut app = self.app.lock().unwrap();
            let _res = action::add_region(&mut app, self.param.0.clone(), faderegion_p);
        }
        self.regions = get_region_from_param(&self.param);
    }
}

impl egui::Widget for &mut Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (bg_response, bg_painter) = ui.allocate_painter(
            egui::vec2(ui.available_width(), gui::TRACK_HEIGHT + 30.0),
            egui::Sense::hover(),
        );
        if bg_response.hovered() {
            bg_painter.rect(
                bg_response.rect,
                2.0,
                ui.style().visuals.faint_bg_color,
                egui::Stroke::new(2.0, egui::Color32::DARK_GRAY),
            );
        }
        let response = ui
            .allocate_ui_at_rect(bg_response.rect, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_width(ui.available_width() * 0.9); //tekitou
                    ui.set_min_height(gui::TRACK_HEIGHT); //tekitou

                    let mut stroke = ui.style().visuals.window_stroke();
                    stroke.width = 0.0;
                    ui.style_mut().visuals.widgets.noninteractive.bg_stroke = stroke;
                    let _group = ui.group(|ui| {
                        {
                            for region in self.regions.iter_mut() {
                                let top = ui.available_rect_before_wrap().top();
                                let x_start = region.params.range.start() as f32
                                    / gui::SAMPLES_PER_PIXEL_DEFAULT;
                                let x_end = region.params.range.end() as f32
                                    / gui::SAMPLES_PER_PIXEL_DEFAULT;
                                let rect = egui::Rect::from_points(&[
                                    [x_start, top].into(),
                                    [x_end, top + gui::TRACK_HEIGHT].into(),
                                ]);
                                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                                ui.put(rect, region);
                            }
                        } //first lock drops here
                    });
                    ui.allocate_ui(ui.min_size(), |ui| {
                        ui.horizontal_centered(|ui| {
                            if ui.button("+").clicked() {
                                self.add_region();
                            }
                        });
                    });
                })
                .response;
            })
            .response;
        response
    }
}
