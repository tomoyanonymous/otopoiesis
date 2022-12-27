use crate::utils::atomic::{self, SimpleAtomic};
use std::ops::RangeInclusive;
use std::sync::Arc;
pub(super) enum HandleMode {
    Start,
    End,
}
impl From<bool> for HandleMode {
    fn from(is_start: bool) -> Self {
        match is_start {
            true => Self::Start,
            false => Self::End,
        }
    }
}
pub(super) struct UiBarState {
    saved_state: i64,
    range: RangeInclusive<i64>,
}
impl UiBarState {
    pub fn new(range: RangeInclusive<i64>) -> Self {
        Self {
            saved_state: 0,
            range,
        }
    }
}

pub(super) struct UiBar<'a> {
    pos: &'a Arc<atomic::I64>,
    state: &'a mut UiBarState,
    mode: HandleMode,
}
impl<'a> UiBar<'a> {
    pub fn new(pos: &'a Arc<atomic::I64>, state: &'a mut UiBarState, mode: HandleMode) -> Self {
        Self { pos, state, mode }
    }
    pub fn set_limit(&mut self, range: RangeInclusive<i64>) {
        self.state.range = range;
    }
    fn react(&mut self, response: &egui::Response) {
        if response.drag_started() {
            self.state.saved_state = self.pos.load() as i64;
        }
        if response.dragged() {
            self.state.saved_state +=
                (response.drag_delta().x * crate::gui::SAMPLES_PER_PIXEL_DEFAULT) as i64;
            self.pos.store((self.state.saved_state).clamp(
                *self.state.range.start() as i64,
                *self.state.range.end() as i64,
            ) as i64);
        }
        if response.drag_released() {
            self.state.saved_state = 0
        }
    }
}

impl<'a> egui::Widget for UiBar<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
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
