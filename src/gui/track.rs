use crate::action;
use crate::data;
use crate::gui;
use crate::utils::AtomicRange;

use std::sync::{Arc, Mutex};

pub struct Model {
    pub param: data::Track,
    app: Arc<Mutex<data::AppModel>>,
    regions: Vec<gui::region::Model>,
    new_array_count: u32,
}
fn get_region_from_param(track: &data::Track) -> Vec<gui::region::Model> {
    match track {
        data::Track::Regions(regions) => regions
            .lock()
            .unwrap()
            .iter()
            .map(|region| gui::region::Model::new(region.clone(), region.label.clone()))
            .collect::<Vec<_>>(),
        data::Track::Generator(_) => todo!(),
        data::Track::Transformer() => todo!(),
    }
}

impl Model {
    pub fn new(param: data::Track, app: Arc<Mutex<data::AppModel>>) -> Self {
        let regions = get_region_from_param(&param);
        Self {
            param,
            app,
            regions,
            new_array_count: 5,
        }
    }
    fn get_position_to_add(&self) -> u64 {
        self.regions
            .iter()
            .fold(0u64, |acc, region| acc.max(region.params.range.end()))
    }
    fn add_region_to_app(&self, region: Arc<data::Region>) {
        let mut app = self.app.try_lock().unwrap();
        match &self.param {
            data::Track::Regions(regions) => {
                let _res =
                    action::add_region(&mut app, regions.clone(), region);
            }
            data::Track::Generator(_) => todo!(),
            data::Track::Transformer() => todo!(),
        }
    }
    fn add_region(&mut self) {
        let pos = self.get_position_to_add();
        let label = format!("region{}", self.regions.len() + 1);
        let region_param = Arc::new(data::Region::new(
            AtomicRange::from(pos..pos + 49000),
            data::Content::Generator(Arc::new(data::Generator::default())),
            label,
        ));
        let faderegion_p = data::Region::with_fade(region_param);
        self.add_region_to_app(faderegion_p);
        self.regions = get_region_from_param(&self.param);
    }
    fn add_region_array(&mut self, count: u32) {
        let pos = self.get_position_to_add();
        let label = format!("region{}", self.regions.len() + 1);

        let region_elem = Arc::new(data::Region::new(
            AtomicRange::from(pos..pos + 4000),
            data::Content::Generator(Arc::new(data::Generator::default())),
            label.clone(),
        ));

        let region_array = Arc::new(data::Region::new(
            AtomicRange::from(pos..pos + 49000),
            data::Content::Transformer(
                Arc::new(data::RegionFilter::Replicate(Arc::new(count.into()))),
                data::Region::with_fade(region_elem),
            ),
            label,
        ));
        self.add_region_to_app(region_array);
        self.regions = get_region_from_param(&self.param);
    }
}

impl egui::Widget for &mut Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let height = gui::TRACK_HEIGHT + 30.0;
        let res = ui.allocate_ui(egui::vec2(ui.available_size().x, height), |ui| {
            let area = ui.available_rect_before_wrap();
            let left_top = area.left_top();
            let top = area.top() - height / 2.;
            let res = ui.group(|ui| {
                self.regions
                    .iter_mut()
                    .map(|region| {
                        let range = region.params.range.clone();
                        let x_start = area.left() + range.start() as f32 / gui::SAMPLES_PER_PIXEL_DEFAULT;
                        let x_end = area.left() + range.end() as f32 / gui::SAMPLES_PER_PIXEL_DEFAULT;
                        let rect = egui::Rect::from_points(&[
                            [x_start, top].into(),
                            [x_end, top + height].into(),
                        ]);
                        ui.put(rect, region)
                    })
                    .last()
            });
            let button_w = 40.0;
            let rect_right = res.inner.map_or_else(
                || {
                    egui::Rect::from_center_size(
                        left_top + egui::vec2(button_w * 0.5, gui::TRACK_HEIGHT * 0.5),
                        [button_w, gui::TRACK_HEIGHT].into(),
                    )
                },
                |inner| inner.rect,
            );
            let new_rect = egui::Rect::from_center_size(
                rect_right.right_center() + egui::vec2(10.0, 0.0),
                egui::vec2(button_w, gui::TRACK_HEIGHT),
            );
            let _buttons = ui.allocate_ui_at_rect(new_rect, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.button("+").clicked() {
                        self.add_region();
                    }
                    ui.horizontal_centered(|ui| {
                        if ui.button("+…").clicked() {
                            self.add_region_array(self.new_array_count);
                        }
                        let _ = ui.add(egui::DragValue::new(&mut self.new_array_count));
                    })
                })
            });
        });

        res.response
    }
}
