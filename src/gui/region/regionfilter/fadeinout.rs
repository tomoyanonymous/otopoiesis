use crate::data;
use crate::gui;
use crate::utils::{AtomicRange, SimpleAtomic};

/// origin needed to be boxed to be recursive data structure.

pub struct State {
    pub origin: Box<super::region::State>,
    pub range: AtomicRange<f64>,
    start_tmp: f32,
    end_tmp: f32,
}
impl State {
    pub fn new(origin: &data::Region, range: AtomicRange<f64>) -> Self {
        let label = &origin.label.clone();
        Self {
            origin: Box::new(super::region::State::new(
                origin,
                format!("{}_fade", label),
                false,
            )),
            range,
            start_tmp: 0.0,
            end_tmp: 0.0,
        }
    }
}

pub struct FadeHandle<'a> {
    param: &'a data::FadeParam,
    origin_ui: &'a mut data::Region,
    state: &'a mut State,
}
impl<'a> FadeHandle<'a> {
    pub fn new(
        param: &'a data::FadeParam,
        origin_ui: &'a mut data::Region,
        state: &'a mut State,
    ) -> Self {
        Self {
            param,
            origin_ui,
            state,
        }
    }
}

impl<'a> egui::Widget for FadeHandle<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let origin = ui.add(super::region::Model::new(
            self.origin_ui,
            self.state.origin.as_mut(),
        ));

        let target_rect = origin.rect;
        let _response = ui.allocate_rect(target_rect, egui::Sense::focusable_noninteractive());

        let mut make_circle = |w, is_start: bool| {
            let radius = 5.0;
            let top = target_rect.top();
            let (rect, handle_pos) = if is_start {
                let handle_pos = egui::pos2(target_rect.left() + w, top + radius * 2.0);
                let rect = egui::Rect::from_points(&[target_rect.left_bottom(), handle_pos]);
                (rect, handle_pos)
            } else {
                let handle_pos = egui::pos2(target_rect.right() - w, top + radius * 2.0);
                let rect = egui::Rect::from_points(&[handle_pos, target_rect.right_bottom()]);
                (rect, handle_pos)
            };
            let handle_pos_another = if is_start {
                handle_pos + egui::vec2(-radius, radius)
            } else {
                handle_pos + egui::vec2(radius, radius)
            };
            let handle_area = egui::Rect::from_two_pos(handle_pos, handle_pos_another);
            let ui_handle = ui.allocate_rect(handle_area, egui::Sense::click_and_drag());

            let painter = ui.painter_at(rect);

            let c = egui::Color32::DARK_GRAY;
            let points = if is_start {
                [rect.left_bottom(), handle_pos]
            } else {
                [handle_pos, rect.right_bottom()]
            };

            painter.line_segment(points, egui::Stroke::new(1.0, c));
            painter.rect_filled(handle_area, 1.0, egui::Color32::DARK_GRAY);

            ui_handle.on_hover_cursor(egui::CursorIcon::PointingHand)
        };
        let range_sec = self.state.range.end() - self.state.range.start();
        let range_pix = range_sec as f32 * gui::PIXELS_PER_SEC_DEFAULT;
        let sec_to_pix = move |sec| sec * gui::PIXELS_PER_SEC_DEFAULT;
        let pix_to_sec = move |pix| pix / gui::PIXELS_PER_SEC_DEFAULT;
        let time_in = self.param.time_in.load();
        let time_out = self.param.time_out.load();

        // to prevent crossing fade handles when the region length was modified.
        if time_in + time_out > range_sec as f32 {
            let delta = time_in + time_out - range_sec as f32;
            self.param.time_in.store(time_in - delta / 2.0);
            self.param.time_out.store(time_out - delta / 2.0);
        }
        let in_width = sec_to_pix(self.param.time_in.load());
        let out_width = sec_to_pix(self.param.time_out.load());
        let start = make_circle(in_width, true);
        if start.drag_started() {
            self.state.start_tmp = in_width;
        }
        if start.dragged() {
            self.state.start_tmp += start.drag_delta().x;
            let bound = range_pix - out_width;
            let v = (self.state.start_tmp + start.drag_delta().x).clamp(0.0, bound);
            self.param.time_in.store(pix_to_sec(v));
        }
        if start.drag_released() {
            self.state.start_tmp = 0.0;
        }
        let end = make_circle(out_width, false);
        if end.drag_started() {
            self.state.end_tmp = out_width;
        }
        if end.dragged() {
            //accumlate delta.
            self.state.end_tmp -= end.drag_delta().x;
            let v = (self.state.end_tmp - end.drag_delta().x).clamp(0.0, range_pix - in_width);
            self.param.time_out.store(pix_to_sec(v));
        }
        if end.drag_released() {
            self.state.end_tmp = 0.0;
        }
        start.union(end)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn scaling() {
        let sec_to_pix = move |sec| sec * gui::PIXELS_PER_SEC_DEFAULT;
        let pix_to_sec = move |pix| pix / gui::PIXELS_PER_SEC_DEFAULT;
        let input_sec: f32 = 30.0;
        assert_eq!(input_sec, pix_to_sec(sec_to_pix(input_sec)));
    }
}
