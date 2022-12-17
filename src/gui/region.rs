use egui::Widget;

use crate::data;
use crate::gui;

use crate::utils::atomic;
use std::ops::RangeInclusive;
use std::sync::Arc;
pub mod regionfilter;

enum HandleMode {
    Start,
    End,
}
struct UiBar {
    state: Arc<atomic::U64>,
    saved_state: i64,
    range: RangeInclusive<u64>,
    mode: HandleMode,
}
impl UiBar {
    pub fn new(state: Arc<atomic::U64>, mode: HandleMode) -> Self {
        let init = state.load();
        Self {
            state,
            saved_state: 0,
            range: 0..=init,
            mode,
        }
    }
    fn set_limit(&mut self, range: RangeInclusive<u64>) {
        self.range = range;
    }
    fn react(&mut self, response: &egui::Response) {
        if response.drag_started() {
            self.saved_state = self.state.load() as i64;
        }
        if response.dragged() {
            self.saved_state += (response.drag_delta().x * gui::SAMPLES_PER_PIXEL_DEFAULT) as i64;
            self.state
                .store((self.saved_state as u64).clamp(*self.range.start(), *self.range.end()));
        }
        if response.drag_released() {
            self.saved_state = 0
        }
    }
}

impl egui::Widget for &mut UiBar {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        let icon = match self.mode {
            HandleMode::Start => egui::CursorIcon::ResizeWest,
            HandleMode::End => egui::CursorIcon::ResizeEast,
        };
        response = response.on_hover_cursor(icon);
        if response.hovered() {
            painter.rect_filled(rect, 0., ui.style().visuals.weak_text_color());
        }

        if response.dragged() {
            painter.rect_filled(rect, 0., ui.style().visuals.strong_text_color());
            ui.ctx().output().cursor_icon = icon;
        }

        self.react(&response);
        let rect_x = ui.min_rect().left();
        let _text = response
            .hover_pos()
            .map_or("none".to_string(), |p| format!("{:?}/offset:{}", p, rect_x));
        response
    }
}

pub enum ContentModel {
    RegionFilter(regionfilter::RegionFilter),
    Generator(super::generator::Generator),
    Array(Vec<Model>),
}

fn ui_vec<'a, T>(vec: &'a mut Vec<T>, ui: &mut egui::Ui) -> egui::Response
where
    T: 'a,
    &'a mut T: egui::Widget,
{
    vec.iter_mut()
        .map(|r| r.ui(ui))
        .fold(None, |acc: Option<egui::Response>, response| {
            acc.map_or_else(
                || Some(response.clone()),
                |a| Some(response.union(a.clone())),
            )
        })
        .unwrap_or(ui.group(|_| {}).response)
}

impl egui::Widget for &mut ContentModel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            ContentModel::RegionFilter(p) => p.ui(ui),
            ContentModel::Generator(g) => g.ui(ui),
            ContentModel::Array(vec) => ui_vec(vec, ui),
        }
    }
}

pub struct Model {
    pub params: Arc<data::Region>,
    pub label: String,
    content: ContentModel,
    range_handles: [UiBar; 2], // pub osc_params: Arc<oscillator::SharedParams>,
    offset_saved: i64,
}

impl Model {
    pub fn new(params: Arc<data::Region>, labeltext: impl ToString) -> Self {
        let label = labeltext.to_string();
        let handle_left = UiBar::new(params.range.0.clone(), HandleMode::Start);
        let handle_right = UiBar::new(params.range.1.clone(), HandleMode::End);
        let content = match &params.content {
            data::Content::Generator(param) => {
                ContentModel::Generator(super::generator::Generator::new(param.clone()))
            }
            data::Content::AudioFile(_) => todo!(),
            data::Content::Transformer(filter, origin) => ContentModel::RegionFilter(
                regionfilter::RegionFilter::new(filter.clone(), origin.clone()),
            ),
            data::Content::Array(vec) => ContentModel::Array(
                vec.iter()
                    .map(|region| Self::new(region.clone(), region.label.clone()))
                    .collect(),
            ),
        };
        Self {
            label,
            content,
            params,
            range_handles: [handle_left, handle_right],
            offset_saved: 0,
        }
    }
    pub fn get_current_amp(&self) -> f32 {
        // self.osc_params.amp.get().abs()
        1.0
    }
    fn draw_main(&mut self, ui: &mut egui::Ui, is_interactive: bool) -> egui::Response {
        let mut main = ui.add(&mut self.content);
        if is_interactive && main.drag_started() {
            self.offset_saved = self.params.range.start() as i64;
        }
        if is_interactive && main.dragged() {
            let offset = (main.drag_delta().x * gui::SAMPLES_PER_PIXEL_DEFAULT) as i64;
            self.params.range.shift_bounded(offset);
            main = main.on_hover_cursor(egui::CursorIcon::Grabbing)
        }
        if is_interactive && main.drag_released() {
            self.offset_saved = 0;
        }
        main
    }
}

impl std::hash::Hash for Model {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state)
    }
}

impl egui::Widget for &mut Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let height = gui::TRACK_HEIGHT;

        let bar_width = 5.;
        let start = self.params.range.start();
        let end = self.params.range.end();
        let min_start = 0u64;
        let max_end = std::u64::MAX;
        self.range_handles[0].set_limit(min_start..=end);
        self.range_handles[1].set_limit(start..=max_end);
        ui.spacing_mut().item_spacing = egui::vec2(0., 0.);

        ui.horizontal(|ui| {
            let bar_size = egui::vec2(bar_width, height);

            match &self.content {
                // you should not call ui.add(&mut self.content) directly here.
                ContentModel::RegionFilter(f) => {
                    match f {
                        regionfilter::RegionFilter::FadeInOut(f) => {
                            self.params.range.set_start(f.range.start());
                            self.params.range.set_end(f.range.end());
                        }
                        regionfilter::RegionFilter::Replicate(_) => unreachable!(
                            "\"Replicate\" operation must be interpreted before ui evaluation."
                        ),
                    };
                    self.draw_main(ui, false)
                }
                ContentModel::Generator(_) => {
                    ui.add_sized(bar_size, &mut self.range_handles[0]);
                    let main = self.draw_main(ui, true);
                    ui.add_sized(bar_size, &mut self.range_handles[1]);
                    main
                }
                ContentModel::Array(_) => self.draw_main(ui, false),
            }
        })
        .response
    }
}
