use crate::data;
use crate::gui;
use crate::utils::atomic::SimpleAtomic;
mod region_handle;
pub mod regionfilter;
use region_handle::{HandleMode, UiBar, UiBarState};

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
}

impl State {
    pub fn new(params: &data::Region, labeltext: impl ToString) -> Self {
        let handle_left = UiBarState::new(0..=params.range.0.load());
        let handle_right = UiBarState::new(params.range.1.load()..=i64::MAX);
        let content = match &params.content {
            data::Content::Generator(param) => {
                ContentModel::Generator(param.clone(), super::generator::State::new(512))
            }
            data::Content::AudioFile(_) => todo!(),
            data::Content::Transformer(filter, origin) => ContentModel::RegionFilter(
                regionfilter::RegionFilterState::new(filter.clone(), *origin.clone()),
            ),
        };
        let range_handles = [handle_left, handle_right];
        Self {
            label: labeltext.to_string(),
            content,
            range_handles,
            offset_saved: 0,
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
    fn interact_main(&mut self, main: &egui::Response, is_interactive: bool) -> egui::Response {
        let mut main = main.clone();
        if is_interactive && main.drag_started() {
            self.state.offset_saved = self.params.range.start() as i64;
        }
        if is_interactive && main.dragged() {
            let offset = (main.drag_delta().x * gui::SAMPLES_PER_PIXEL_DEFAULT) as i64;
            self.params.range.shift_bounded(offset);
            main = main.on_hover_cursor(egui::CursorIcon::Grabbing)
        }
        if is_interactive && main.drag_released() {
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
        let min_start = 0;
        let max_end = end + self.params.range.getrange();

        ui.spacing_mut().item_spacing = egui::vec2(0., 0.);

        ui.horizontal(|ui| {
            let bar_size = egui::vec2(bar_width, height);

            let (main, is_interactive) = match (&mut self.params.content, &mut self.state.content) {
                (data::Content::Transformer(filter, origin), ContentModel::RegionFilter(f)) => {
                    let fade_param = match filter {
                        data::RegionFilter::Gain => todo!(),
                        data::RegionFilter::FadeInOut(param) => {
                            self.params.range.set_start(origin.range.start());
                            self.params.range.set_end(origin.range.end());
                            param
                        }
                        data::RegionFilter::Reverse => todo!(),
                        data::RegionFilter::Replicate(_) => todo!(),
                    };

                    (
                        ui.add(regionfilter::RegionFilter::new(
                            fade_param,
                            f,
                            origin.as_mut(),
                        )),
                        false,
                    )
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

            self.interact_main(&main, is_interactive)
        })
        .response
    }
}

/// UI model that is used only for display element.
/// it is mostly for displaying information for replicating regions.
/// Any data are not shared between audio thread.
pub(crate) struct ReadOnlyModel {
    // params: data::Region,
    content: ContentModel,
}
impl ReadOnlyModel {
    pub fn new(origin: &data::Region) -> Self {
        // let params = data::Region::new(
        //     AtomicRange<i64>::new(origin.range.start(), origin.range.end()),
        //     origin.content.clone(),
        //     "dummy",
        // );
        match &origin.content {
            data::Content::Generator(g) => Self {
                // params,
                content: ContentModel::Generator(g.clone(), super::generator::State::new(512)),
            },
            data::Content::AudioFile(_) => todo!(),
            data::Content::Transformer(filter, origin) => Self {
                content: ContentModel::RegionFilter(regionfilter::RegionFilterState::new(
                    filter.clone(),
                    *origin.clone(),
                )),
            },
            // data::Content::Array(_) => todo!(),
        }
    }
}

impl egui::Widget for &mut ReadOnlyModel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match &mut self.content {
            ContentModel::RegionFilter(_f) => todo!(), //ui.add(f),
            ContentModel::Generator(p, vec) => ui.add(super::generator::Generator::new(p, vec)),
        }
    }
}
