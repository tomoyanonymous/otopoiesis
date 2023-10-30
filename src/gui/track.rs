use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::action;
use crate::action::Action;
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

    fn get_position_to_add(param: &[data::Region]) -> f64 {
        param
            .iter()
            .fold(0.0, |acc, region| acc.max(region.range.end()))
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
    fn add_region(&self, id: usize, generator: data::Generator) {
        let region = if let data::Track::Regions(ref target) = self.track {
            let pos = Self::get_position_to_add(target.as_slice());
            let label = format!("region{}", id + 1);
            let region_param = data::Region::new(
                AtomicRange::<f64>::from(pos..pos + 1.0),
                data::Content::Generator(generator),
                label,
            );
            data::Region::with_fade(region_param)
        } else {
            unreachable!()
        };
        self.action_tx
            .send(Action::from(action::AddRegion::new(region, id)));
    }
    fn add_region_osc(&self, id: usize) {
        self.add_region(id, data::Generator::default());
    }

    #[allow(unused_variables)]
    fn add_regionfile(&self, id: usize) {
        #[cfg(not(feature = "web"))]
        {
            let (file, _len) = data::generator::FilePlayerParam::new_test_file();
            self.add_region(id, data::Generator::FilePlayer(Arc::new(file)));
        }
    }
    fn add_region_array(&self, id: usize, count: u32) {
        let region_array = if let data::Track::Regions(ref target) = self.track {
            let pos = Self::get_position_to_add(target);
            let t_count = target.len() + 1;
            let label = format!("region{}", t_count);
            let region_elem = data::Region::new(
                AtomicRange::<f64>::from(pos..pos + 1.0),
                data::Content::Generator(data::Generator::default()),
                label.clone(),
            );

            data::Region::new(
                AtomicRange::<f64>::from(pos..pos + 1.0),
                data::Content::Transformer(
                    data::RegionFilter::Replicate(count.into()),
                    Box::new(data::Region::with_fade(region_elem)),
                ),
                label,
            )
        } else {
            unreachable!()
        };
        self.action_tx
            .send(Action::from(action::AddRegion::new(region_array, id)));
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
                let regions = ui.allocate_ui(egui::vec2(w, height), |ui| {

                    let area = ui.available_rect_before_wrap();
                    ui.set_min_width(100.);
                    ui.set_min_height(gui::TRACK_HEIGHT);
                    let top = area.top();
                    let scale = move |sec: f64| (sec * gui::PIXELS_PER_SEC_DEFAULT as f64) as f32;
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
                let button_w = 40.0;
                let new_rect = egui::Rect::from_center_size(
                    regions.response.rect.right_center() + egui::vec2(10.0, 0.0),
                    egui::vec2(button_w, gui::TRACK_HEIGHT),
                );

        // ui.painter().rect_filled(new_rect, 0.0, egui::Color32::BLUE);

                let menu = ui.allocate_ui_at_rect(new_rect, |ui| {
                    ui.set_min_width(100.);
                    ui.set_min_height(gui::TRACK_HEIGHT);
                    // let response =
                    //     ui.allocate_response([100.0, gui::TRACK_HEIGHT].into(), egui::Sense::click());
                    // response
                    let menues = ui.horizontal_centered(|ui| {
                        egui::menu::bar(ui, |ui| {
                            let button = ui
                                .menu_button("+", |ui| {
                                    let addregion = ui.button("~ Add oscillator");

                                    let addfile = ui.button("💾 Load File");
                                    let addarray = ui
                                        .horizontal(|ui| {
                                            let res = ui.button("~…");
                                            let _ = ui.add(egui::DragValue::new(
                                                &mut self.state.new_array_count,
                                            ));
                                            res
                                        })
                                        .inner;
                                    if addregion.clicked() {
                                        self.add_region_osc(self.id);
                                    }
                                    if addfile.clicked() {
                                        self.add_regionfile(self.id);
                                    }
                                    if addarray.clicked() {
                                        self.add_region_array(self.id, self.state.new_array_count);
                                    }
                                    (addregion, addfile, addarray)
                                })
                                .inner;
                            button
                        });
                    });
                    menues.inner
                });

                // if let Some((addregion, addfile, addarray)) = res.inner {
                //     if addregion.clicked() {
                //         self.add_region_osc(self.id);
                //     }
                //     if addfile.clicked() {
                //         self.add_regionfile(self.id);
                //     }
                //     if addarray.clicked() {
                //         self.add_region_array(self.id, self.state.new_array_count);
                //     }
                // }
                if menu.response.clicked() {
                    self.sync_state();
                }
                regions.response.union(menu.response)
            }

            data::Track::Generator(_) => todo!(),
            data::Track::Transformer() => todo!(),
        };

        response
    }
}
