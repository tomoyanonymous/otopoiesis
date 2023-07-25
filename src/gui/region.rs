use crate::data;
use crate::gui;
use crate::utils::atomic::SimpleAtomic;
mod region_handle;
pub mod regionfilter;
use region_handle::{HandleMode, UiBar, UiBarState};

use self::regionfilter::fadeinout::FadeHandle;
use self::regionfilter::replicate::Replicate;
use self::regionfilter::RegionFilterState;
use self::regionfilter::{fadeinout, replicate};

pub enum ContentModel {
    RegionFilter(regionfilter::RegionFilterState),
    Generator(data::Generator, super::generator::State),
}

// fn ui_vec<'a, T>(vec: &'a mut [T], ui: &mut egui::Ui) -> egui::Response
// where
//     T: 'a,
//     &'a mut T: egui::Widget,
// {
//     ui.group(|ui| {
//         vec.iter_mut()
//             .map(|r| r.ui(ui))
//             .fold(None, |acc: Option<egui::Response>, response| {
//                 acc.map_or_else(|| Some(response.clone()), |a| Some(response.union(a)))
//             })
//             .unwrap_or(ui.group(|_| {}).response)
//     })
//     .inner
// }

pub struct State {
    pub label: String,
    content: ContentModel,
    range_handles: [UiBarState; 2], // pub osc_params: Arc<oscillator::SharedParams>,
    offset_saved: i64,
    #[allow(dead_code)]
    is_interactive: bool,
}

impl State {
    pub fn new(params: &data::Region, labeltext: impl ToString, is_interactive: bool) -> Self {
        let handle_left = UiBarState::new(0.0..=params.range.0.load());
        let handle_right = UiBarState::new(params.range.1.load()..=f64::MAX);
        let content = match &params.content {
            data::Content::Generator(param) => {
                ContentModel::Generator(param.clone(), super::generator::State::new(512))
            }
            data::Content::Transformer(filter, origin) => {
                ContentModel::RegionFilter(match filter {
                    data::RegionFilter::Gain => todo!(),
                    data::RegionFilter::Reverse => todo!(),
                    data::RegionFilter::FadeInOut(_p) => {
                        regionfilter::RegionFilterState::FadeInOut(fadeinout::State::new(
                            origin,
                            origin.range.clone(),
                        ))
                    }
                    data::RegionFilter::Replicate(p) => regionfilter::RegionFilterState::Replicate(
                        replicate::State::new(origin.as_ref(), p.count.load() as u64),
                    ),
                })
            }
        };
        let range_handles = [handle_left, handle_right];
        Self {
            label: labeltext.to_string(),
            content,
            range_handles,
            offset_saved: 0,
            is_interactive,
        }
    }
}

pub struct Model<'a> {
    pub params: &'a mut data::Region,
    pub state: &'a mut State,
}

impl<'a> Model<'a> {
    pub fn new(params: &'a mut data::Region, state: &'a mut State) -> Self {
        Self { params, state }
    }
    pub fn get_current_amp(&self) -> f32 {
        // self.osc_params.amp.get().abs()
        1.0
    }
    fn interact_main(&mut self, main: &egui::Response) -> egui::Response {
        let mut main = main.clone();
        if main.drag_started() {
            self.state.offset_saved = self.params.range.start() as i64;
        }
        if main.dragged() {
            let offset = main.drag_delta().x as f64 / gui::PIXELS_PER_SEC_DEFAULT as f64;
            self.params.range.shift(offset);
            main = main.on_hover_cursor(egui::CursorIcon::Grabbing)
        }
        if main.drag_released() {
            self.state.offset_saved = 0;
        }
        main
    }
}

impl<'a> std::hash::Hash for Model<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.label.hash(state)
    }
}

impl<'a> egui::Widget for Model<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let height = gui::TRACK_HEIGHT;

        let bar_width = 5.;
        let start = self.params.range.start();
        let end = self.params.range.end();
        let min_start = 0.0;
        let max_end = end + self.params.range.getrange();

        ui.spacing_mut().item_spacing = egui::vec2(0., 0.);

        ui.horizontal(|ui| {
            let bar_size = egui::vec2(bar_width, height);

            let (main, is_interactive) = match (&mut self.params.content, &mut self.state.content) {
                (data::Content::Transformer(filter, origin), ContentModel::RegionFilter(state)) => {
                    match (filter, state) {
                        (data::RegionFilter::Gain, _) => todo!(),
                        (data::RegionFilter::FadeInOut(param), RegionFilterState::FadeInOut(s)) => {
                            self.params.range.set_start(origin.range.start());
                            self.params.range.set_end(origin.range.end());
                            (
                                ui.add(regionfilter::RegionFilter::FadeInOut(FadeHandle::new(
                                    param.as_ref(),
                                    origin.as_mut(),
                                    s,
                                ))),
                                false,
                            )
                        }
                        (data::RegionFilter::Reverse, _) => todo!(),
                        (data::RegionFilter::Replicate(param), RegionFilterState::Replicate(s)) => {
                            (
                                ui.add(regionfilter::RegionFilter::Replicate(Replicate::new(
                                    param,
                                    origin.as_mut(),
                                    s,
                                ))),
                                false,
                            )
                        }
                        (_, _) => panic!(
                            "invalid combination of parameter and gui state in pattern matching "
                        ),
                    }
                }
                (data::Content::Generator(param), ContentModel::Generator(_genmodel, genstate)) => {
                    let mut handle_start = UiBar::new(
                        &self.params.range.0,
                        &mut self.state.range_handles[0],
                        HandleMode::Start,
                    );
                    handle_start.set_limit(min_start..=end);
                    ui.add_sized(bar_size, handle_start);
                    let main = ui.add(super::generator::Generator::new(param, genstate));
                    let mut handle_end = UiBar::new(
                        &self.params.range.1,
                        &mut self.state.range_handles[1],
                        HandleMode::End,
                    );
                    handle_end.set_limit(start..=max_end);
                    ui.add_sized(bar_size, handle_end);
                    (main, true)
                }
                _ => unreachable!(),
            };

            if  is_interactive {
                self.interact_main(&main);
            }
        })
        .response
    }
}
