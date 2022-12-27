use std::sync::Arc;
use std::sync::Mutex;

use egui::InnerResponse;

use crate::action;
use crate::data;
use crate::gui;
use crate::utils::AtomicRange;

pub struct State {
    regions: Vec<gui::region::State>,
    new_array_count: u32,
}
impl State {
    pub fn new(param: &data::Track, new_array_count: u32) -> Self {
        let regions = get_region_from_param(param);

        Self {
            regions,
            new_array_count,
        }
    }
}

pub struct Model<'a> {
    id: usize,
    app: Arc<Mutex<data::AppModel>>,
    state: &'a mut State,
}

fn get_region_from_param(track: &data::Track) -> Vec<gui::region::State> {
    match track {
        data::Track::Regions(regions) => regions
            .iter()
            .map(|region| gui::region::State::new(region, region.label.clone()))
            .collect::<Vec<_>>(),
        data::Track::Generator(_) => todo!(),
        data::Track::Transformer() => todo!(),
    }
}

impl<'a> Model<'a> {
    pub fn new(id: usize, app: Arc<Mutex<data::AppModel>>, state: &'a mut State) -> Self {
        Self { id, app, state }
    }

    fn get_position_to_add(param: &[data::Region]) -> i64 {
        param
            .iter()
            .fold(0i64, |acc, region| acc.max(region.range.end()))
    }
    // fn add_region_to_app(&self, region: data::Region) {
    //     match &self.param {
    //         data::Track::Regions(_regions) => {
    //             let _res = action::add_region(&mut self.app.lock().unwrap(), self.state.id, region);
    //         }
    //         data::Track::Generator(_) => todo!(),
    //         data::Track::Transformer() => todo!(),
    //     }
    // }
    fn add_region(app: &mut data::AppModel, id: usize) {
        let region = if let data::Track::Regions(target) = app.get_track_for_id(id).unwrap() {
            let pos = Self::get_position_to_add(target);
            let label = format!("region{}", id + 1);
            let region_param = data::Region::new(
                AtomicRange::<i64>::from(pos..pos + 49000),
                data::Content::Generator(data::Generator::default()),
                label,
            );
            data::Region::with_fade(region_param)
        } else {
            unreachable!()
        };
        action::add_region(app, id, region).unwrap();
    }

    fn add_region_array(app: &mut data::AppModel, id: usize, count: u32) {
        let region_array = if let data::Track::Regions(target) = app.get_track_for_id(id).unwrap() {
            let pos = Self::get_position_to_add(target);
            let t_count = target.len() + 1;
            let label = format!("region{}", t_count);
            let region_elem = data::Region::new(
                AtomicRange::<i64>::from(pos..pos + 4000),
                data::Content::Generator(data::Generator::default()),
                label.clone(),
            );

            data::Region::new(
                AtomicRange::<i64>::from(pos..pos + 49000),
                data::Content::Transformer(
                    data::RegionFilter::Replicate(count.into()),
                    Box::new(data::Region::with_fade(region_elem)),
                ),
                label,
            )
        } else {
            unreachable!()
        };
        action::add_region(app, id, region_array).unwrap();
    }
    fn sync_state(&mut self) {
        self.state.regions =
            get_region_from_param(self.app.lock().unwrap().get_track_for_id(self.id).unwrap());
    }
}

impl<'a> egui::Widget for Model<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let height = gui::TRACK_HEIGHT + 30.0;
        let (addregion, addarray, response) = if let Ok(mut app) = self.app.lock() {
            let track = app.get_track_for_id(self.id).unwrap();
            match track {
                data::Track::Regions(region_params) => {
                    let w = ui.available_size().x;
                    let InnerResponse {
                        inner: (addregion, addarray),
                        response,
                    } = ui.allocate_ui(egui::vec2(w, height), |ui| {
                        let area = ui.available_rect_before_wrap();
                        let left_top = area.left_top();
                        let top = area.top() - height / 2.;

                        let res = ui.group(|ui| {
                            self.state
                                .regions
                                .iter_mut()
                                .zip(region_params.iter_mut())
                                .map(|(region, region_param)| {
                                    let range = region_param.range.clone();
                                    let x_start = area.left()
                                        + range.start() as f32 / gui::SAMPLES_PER_PIXEL_DEFAULT;
                                    let x_end = area.left()
                                        + range.end() as f32 / gui::SAMPLES_PER_PIXEL_DEFAULT;
                                    let rect = egui::Rect::from_points(&[
                                        [x_start, top].into(),
                                        [x_end, top + height].into(),
                                    ]);
                                    ui.put(rect, super::region::Model::new(region_param, region))
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
                        let InnerResponse {
                            inner: (addregion, addarray),
                            response: _,
                        } = ui.allocate_ui_at_rect(new_rect, |ui| {
                            ui.vertical_centered(|ui| {
                                let addregion_button = ui.button("+");
                                let addarray_button = ui
                                    .horizontal_centered(|ui| {
                                        let res = ui.button("+…");
                                        let _ = ui.add(egui::DragValue::new(
                                            &mut self.state.new_array_count,
                                        ));
                                        res
                                    })
                                    .inner;

                                (addregion_button, addarray_button)
                            })
                            .inner
                        });
                        (addregion, addarray)
                    });
                    if addregion.clicked() {
                        Self::add_region(&mut app, self.id);
                    }
                    if addarray.clicked() {
                        Self::add_region_array(&mut app, self.id, self.state.new_array_count);
                    }
                    (addregion, addarray, response)
                }

                data::Track::Generator(_) => todo!(),
                data::Track::Transformer() => todo!(),
            }
        } else {
            unreachable!()
        };
        if addregion.clicked() || addarray.clicked() {
            self.sync_state()
        }

        response
    }
}
