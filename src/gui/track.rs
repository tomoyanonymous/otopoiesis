use egui::Response;
use ringbuf::{HeapCons};
use crate::parameter::{FloatParameter, Parameter};

// use crate::action::Action;
use crate::data;
use crate::data::TrackContent;
use crate::gui;
// use crate::gui::menu;
use std::sync::mpsc;

// use super::menu::add_fade_to_region;

pub struct Model<'a> {
    id: usize,
    // action_tx: mpsc::Sender<Action>,
    track: &'a mut data::Track,
}

impl<'a> Model<'a> {
    pub fn new(id: usize, track: &'a mut data::Track) -> Self {
        Self {
            id,
            track,
        }
    }

    fn sync_state(&mut self) {}
}

impl<'a> egui::Widget for Model<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let height = gui::TRACK_HEIGHT + 30.0;

        let response = match &mut self.track.track_content {
            TrackContent::Regions(regions) => {
                let w = ui.available_size().x;
                let top = ui.available_rect_before_wrap().top();

                ui.allocate_ui(egui::vec2(w, height), |ui| {
                    let area = ui.available_rect_before_wrap();
                    ui.set_min_width(100.);
                    ui.set_min_height(gui::TRACK_HEIGHT);
                    let scale = move |sec: f64| (sec * gui::PIXELS_PER_SEC_DEFAULT as f64) as f32;
                    ui.group(|ui| {
                        regions
                            .iter_mut()
                            .enumerate()
                            .map(|(i, region)| {
                                ui.push_id(i, |ui| {
                                    let start = region.start.get();
                                    let end = start + region.dur.get();
                                    let x_start = area.left() + scale(start as _);
                                    let x_end = area.left() + scale(end as _);
                                    let rect = egui::Rect::from_points(&[
                                        [x_start, top].into(),
                                        [x_end, top + height].into(),
                                    ]);
                                    let res = ui.put(rect, super::region::Model::new(region));
                                    // res.context_menu(|ui| {
                                    //     let _ = add_fade_to_region(self.id, i, &self.action_tx, ui);
                                    // })
                                })
                                .inner
                            })
                            .last()
                    })
                })
                .response

                // let new_rect = egui::Rect::from_center_size(
                //     egui::pos2(region_right_x, top + gui::TRACK_HEIGHT / 2.0),
                //     egui::vec2(button_w, gui::TRACK_HEIGHT),
                // );

                // ui.painter().rect_filled(new_rect, 0.0, egui::Color32::BLUE);

                // let menu = ui.scope_builder(egui::UiBuilder::new().max_rect(new_rect), |ui| {
                //     ui.set_min_width(40.);
                //     ui.set_height(gui::TRACK_HEIGHT);
                //     let position = self.get_position_to_add();
                //     ui.centered_and_justified(|ui| {
                //         menu::add_region_button(self.id, position, &self.action_tx, ui);
                //     })
                // });
                // menu.response.conte

                // if menu.response.clicked() {
                //     self.sync_state();
                // }
                // if let Some(regions) = regions_opt {
                //     regions.response.union(menu.response)
                // } else {
                //     menu.response
                // }
            }

            TrackContent::SubTracks(tracks) => tracks
                .iter_mut()
                .enumerate()
                .map(|(i, t)| {
                    let model = Model::new(i, t);
                    model.ui(ui)
                })
                .reduce(|u1, u2| u1.union(u2))
                .unwrap_or_else(|| ui.label("no contents")),
            // TrackContent::Generator(generator) => todo!(),
        };

        response
    }
}
