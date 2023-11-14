use crate::action::Action;
use crate::data;
use crate::gui;
use crate::gui::menu;
use std::sync::mpsc;
pub struct State {
    regions: Vec<gui::region::State>,
    // new_array_count: u32,
}
impl State {
    pub fn new(param: &data::Track, _new_array_count: u32) -> Self {
        let regions = get_region_from_param(param);

        Self {
            regions,
            // new_array_count,
        }
    }
}

pub struct Model<'a> {
    id: usize,
    action_tx: mpsc::Sender<Action>,
    track: &'a mut data::Track,
    state: &'a mut State,
}

fn get_region_from_param(track: &data::Track) -> Vec<gui::region::State> {
    match track {
        data::Track::Regions(regions) => regions
            .iter()
            .map(|region| gui::region::State::new(region, region.label.clone(), true))
            .collect::<Vec<_>>(),
        data::Track::Generator(_) => todo!(),
        data::Track::Transformer() => todo!(),
    }
}

impl<'a> Model<'a> {
    pub fn new(
        id: usize,
        action_tx: mpsc::Sender<Action>,
        track: &'a mut data::Track,
        state: &'a mut State,
    ) -> Self {
        Self {
            id,
            action_tx,
            track,
            state,
        }
    }
    fn get_position_to_add(&self) -> f64 {
        match &self.track {
            data::Track::Regions(r) => r
                .iter()
                .fold(0.0, |acc, region| acc.max(region.range.end())),
            _ => unreachable!(),
        }
    }

    fn sync_state(&mut self) {
        self.state.regions = get_region_from_param(self.track);
    }
}

impl<'a> egui::Widget for Model<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let height = gui::TRACK_HEIGHT + 30.0;
        let response = match self.track {
            data::Track::Regions(ref region_params) => {
                let w = ui.available_size().x;
                let top = ui.available_rect_before_wrap().top();

                let regions_opt = if !self.state.regions.is_empty() {
                    let regions = ui.allocate_ui(egui::vec2(w, height), |ui| {
                        let area = ui.available_rect_before_wrap();
                        ui.set_min_width(100.);
                        ui.set_min_height(gui::TRACK_HEIGHT);
                        let scale =
                            move |sec: f64| (sec * gui::PIXELS_PER_SEC_DEFAULT as f64) as f32;
                        ui.group(|ui| {
                            self.state
                                .regions
                                .iter_mut()
                                .zip(region_params.iter())
                                .map(|(region, region_param)| {
                                    let range = region_param.range.clone();
                                    let x_start = area.left() + scale(range.start());
                                    let x_end = area.left() + scale(range.end());
                                    let rect = egui::Rect::from_points(&[
                                        [x_start, top].into(),
                                        [x_end, top + height].into(),
                                    ]);
                                    ui.put(rect, super::region::Model::new(region_param, region))
                                })
                                .last()
                        })
                    });
                    Some(regions)
                } else {
                    None
                };
                let button_w = 20.0;
                let region_right_x = regions_opt
                    .as_ref()
                    .map_or(30.0, |rs| rs.response.rect.right());
                let new_rect = egui::Rect::from_center_size(
                    egui::pos2(region_right_x, top + gui::TRACK_HEIGHT / 2.0),
                    egui::vec2(button_w, gui::TRACK_HEIGHT),
                );

                ui.painter().rect_filled(new_rect, 0.0, egui::Color32::BLUE);

                let menu = ui.allocate_ui_at_rect(new_rect, |ui| {
                    ui.set_min_width(40.);
                    ui.set_height(gui::TRACK_HEIGHT);
                    let position = self.get_position_to_add();
                    ui.centered_and_justified(|ui| {
                        menu::add_region_button(self.id, position, &self.action_tx, ui);
                    })
                });

                if menu.response.clicked() {
                    self.sync_state();
                }
                if let Some(regions) = regions_opt {
                    regions.response.union(menu.response)
                } else {
                    menu.response
                }
            }

            data::Track::Generator(_) => todo!(),
            data::Track::Transformer() => todo!(),
        };

        response
    }
}
