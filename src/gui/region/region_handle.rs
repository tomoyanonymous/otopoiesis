use crate::gui;
use std::ops::RangeInclusive;
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
    saved_state: f64,
    range: RangeInclusive<f64>,
}
impl UiBarState {
    pub fn new(range: RangeInclusive<f64>) -> Self {
        Self {
            saved_state: 0.0,
            range,
        }
    }
}

pub(super) struct UiBar<'a> {
    pos: &'a mut f64,
    state: &'a mut UiBarState,
    mode: HandleMode,
}
impl<'a> UiBar<'a> {
    pub fn new(pos: &'a mut f64, state: &'a mut UiBarState, mode: HandleMode) -> Self {
        Self { pos, state, mode }
    }
    pub fn set_limit(&mut self, range: RangeInclusive<f64>) {
        self.state.range = range;
    }
    fn react(&mut self, response: &egui::Response) {
        if response.drag_started() {
            self.state.saved_state = *self.pos;
        }
        if response.dragged() {
            self.state.saved_state +=
                (response.drag_delta().x / gui::PIXELS_PER_SEC_DEFAULT) as f64;
            let pos = (self.state.saved_state)
                .clamp(*self.state.range.start(), *self.state.range.end())
                as f64;
            *self.pos = pos;
        }
        if response.drag_released() {
            self.state.saved_state = 0.0
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
            ui.ctx().output_mut(|o| o.cursor_icon = icon);
        }

        self.react(&response);
        let rect_x = ui.min_rect().left();
        let _text = response
            .hover_pos()
            .map_or("none".to_string(), |p| format!("{:?}/offset:{}", p, rect_x));
        response
    }
}
