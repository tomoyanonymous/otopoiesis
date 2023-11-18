use std::sync::Arc;

use crate::data;
use crate::gui;
use crate::gui::TRACK_HEIGHT;
use crate::parameter::FloatParameter;
use crate::parameter::Parameter;
use crate::utils::AtomicRange;

/// origin needed to be boxed to be recursive data structure.

pub struct State {
    pub origin: Box<super::region::State>,
    pub start: Arc<FloatParameter>,
    pub dur: Arc<FloatParameter>,
    start_tmp: f32,
    end_tmp: f32,
}
impl State {
    pub fn new(
        origin: &data::Region,
        start: Arc<FloatParameter>,
        dur: Arc<FloatParameter>,
    ) -> Self {
        let label = &origin.label.clone();
        Self {
            origin: Box::new(super::region::State::new(
                origin,
                format!("{}_fade", label),
                false,
            )),
            start,
            dur,
            start_tmp: 0.0,
            end_tmp: 0.0,
        }
    }
}
pub struct FadeInOut<'a> {
    param: &'a data::FadeParam,
    origin_ui: &'a data::Region,
    state: &'a mut State,
}
impl<'a> FadeInOut<'a> {
    pub fn new(
        param: &'a data::FadeParam,
        origin_ui: &'a data::Region,
        state: &'a mut State,
    ) -> Self {
        Self {
            param,
            origin_ui,
            state,
        }
    }
    fn make_handle(
        &self,
        ui: &mut egui::Ui,
        target_rect: &egui::Rect,
        fade_width: f32,
        is_start: bool,
    ) -> egui::Response {
        let radius = 10.0;
        // ui.painter().debug_rect(*target_rect, egui::Color32::RED, "fade");

        let top = target_rect.top();

        let (paint_rect, handle_pos) = if is_start {
            let left = target_rect.left();
            let left_bottom = egui::pos2(left, target_rect.bottom());
            let handle_pos = egui::pos2(left + fade_width, top);
            let right_up = if fade_width > 10.0 {
                handle_pos
            } else {
                egui::pos2(left + 10.0, top)
            };
            let rect = egui::Rect::from_points(&[right_up, left_bottom]);
            (rect, handle_pos)
        } else {
            let right = target_rect.right();
            let right_bottom = egui::pos2(right, target_rect.bottom());
            let handle_pos = egui::pos2(right - fade_width, top);
            let left_up = if fade_width > 10.0 {
                handle_pos
            } else {
                egui::pos2(right - 10.0, top)
            };
            let rect = egui::Rect::from_points(&[left_up, right_bottom]);
            (rect, handle_pos)
        };

        let paint_rect = paint_rect.translate([0.0, radius].into());
        let handle_area = egui::Rect::from_center_size(
            handle_pos + [0.0, radius].into(),
            [radius, radius].into(),
        );
        // let ui_handle_proto = ui.child_ui(handle_area, egui::Layout::top_down_justified(egui::Align::Center));
        let ui_handle = ui.allocate_rect(handle_area, egui::Sense::click_and_drag());

        // paint_rect.extend_with_x(paint_rect.left() - radius);
        let painter = ui.painter_at(paint_rect);

        let c = egui::Color32::DARK_GRAY;
        let points = if is_start {
            [paint_rect.left_bottom(), handle_area.center()]
        } else {
            [handle_area.center(), paint_rect.right_bottom()]
        };

        painter.line_segment(points, egui::Stroke::new(2.0, c));
        // painter.rect_stroke(paint_rect, 0.0, ui.style().visuals.window_stroke);
        painter.rect_filled(handle_area, 0.0, egui::Color32::RED);
        painter.add(egui::Shape::circle_filled(
            handle_pos,
            radius,
            egui::Color32::RED,
        ));
        ui_handle
    }
}

impl<'a> egui::Widget for FadeInOut<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let origin = ui.add(super::region::Model::new(
            self.origin_ui,
            self.state.origin.as_mut(),
        ));
        let mut target_rect = origin.rect;
        target_rect.set_bottom(target_rect.top() + TRACK_HEIGHT);

        // let _response = ui.allocate_rect(target_rect, egui::Sense::focusable_noninteractive());
        let range_sec = self.state.dur.get();
        let range_pix = range_sec as f32 * gui::PIXELS_PER_SEC_DEFAULT;
        let sec_to_pix = move |sec| sec * gui::PIXELS_PER_SEC_DEFAULT;
        let pix_to_sec = move |pix| pix / gui::PIXELS_PER_SEC_DEFAULT;
        let time_in = self.param.time_in.get();
        let time_out = self.param.time_out.get();

        // to prevent crossing fade handles when the region length was modified.
        if time_in + time_out > range_sec as f32 {
            let delta = time_in + time_out - range_sec as f32;
            self.param.time_in.set(time_in - delta / 2.0);
            self.param.time_out.set(time_out - delta / 2.0);
        }
        let in_width = sec_to_pix(self.param.time_in.get());
        let out_width = sec_to_pix(self.param.time_out.get());

        let start = self.make_handle(ui, &target_rect, in_width, true);
        let start = start.on_hover_cursor(egui::CursorIcon::PointingHand);
        if start.drag_started() {
            self.state.start_tmp = in_width;
        }
        if start.dragged() {
            self.state.start_tmp += start.drag_delta().x;
            let bound = range_pix - out_width;
            let v = (self.state.start_tmp + start.drag_delta().x).clamp(0.0, bound);
            self.param.time_in.set(pix_to_sec(v));
        }
        if start.drag_released() {
            self.state.start_tmp = 0.0;
        }
        let end = self.make_handle(ui, &target_rect, out_width, false);
        let end = end.on_hover_cursor(egui::CursorIcon::PointingHand);
        if end.drag_started() {
            self.state.end_tmp = out_width;
        }
        if end.dragged() {
            //accumlate delta.
            self.state.end_tmp -= end.drag_delta().x;
            let v = (self.state.end_tmp - end.drag_delta().x).clamp(0.0, range_pix - in_width);
            self.param.time_out.set(pix_to_sec(v));
        }
        if end.drag_released() {
            self.state.end_tmp = 0.0;
        }
        // start.union(end)
        // end.union(start)
        start
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
