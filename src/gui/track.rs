use crate::data;
use crate::gui;
use crate::utils::{atomic,AtomicRange};
use crate::action;

use egui;


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
        let (bg_response, bg_painter) = ui.allocate_painter(
            egui::vec2(ui.available_width(), gui::TRACK_HEIGHT+30.0),
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
        let response = ui.allocate_ui_at_rect(bg_response.rect, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.set_min_width(ui.available_width() * 0.9); //tekitou
                ui.set_min_height(gui::TRACK_HEIGHT);//tekitou

                let mut track_len = 0;
                let mut x_rightmost = 0;
                let mut stroke = ui.style().visuals.window_stroke();
                stroke.width = 0.0;
                ui.style_mut().visuals.widgets.noninteractive.bg_stroke = stroke;
                let _group = ui.group(|ui| {
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
                });
                ui.allocate_ui(ui.min_size(), |ui| {
                    ui.horizontal_centered(|ui| {
                        if ui.button("+").clicked() {
                            let label = format!("region{}", track_len).to_string();
                            let region_param = Arc::new(data::Region {
                                range: AtomicRange::new(x_rightmost, x_rightmost + 49000),
                                max_size: atomic::U64::from(60000),
                                generator: Arc::new(data::Generator::Oscillator(Arc::new(
                                    data::OscillatorParam::default(),
                                ))),
                                filters: vec![],
                                label,
                            });
                            let mut app = self.app.lock().unwrap();
                            let _res =
                                action::add_region(&mut app, self.param.0.clone(), region_param);
                        }
                    });
                });
            })
            .response;
        }).response;
        response
    }
}
