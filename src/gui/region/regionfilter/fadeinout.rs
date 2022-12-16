use crate::data;
use crate::gui;
use crate::utils::AtomicRange;
use std::sync::Arc;

/// origin needed to be boxed to be recursive data structure.

pub struct FadeHandle {
    param: Arc<data::FadeParam>,
    origin: Box<super::region::Model>,
    pub range: AtomicRange,
    start_tmp: f32,
    end_tmp: f32,
}
impl FadeHandle {
    pub fn new(
        param: Arc<data::FadeParam>,
        origin: Arc<data::Region>,
        range: &AtomicRange,
    ) -> Self {
        let label = &origin.label.clone();
        Self {
            param,
            origin: Box::new(super::region::Model::new(origin, format!("{}_fade", label))),
            range: range.clone(),
            start_tmp: 0.0,
            end_tmp: 0.0,
        }
    }
}

impl egui::Widget for &mut FadeHandle {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let origin = ui.add(self.origin.as_mut());

        let target_rect = origin.rect;
        let _response = ui.allocate_rect(target_rect, egui::Sense::focusable_noninteractive());

        let mut make_circle = |w, is_start: bool| {
            let radius = 5.0;
            let (rect, handle_pos) = if is_start {
                let handle_pos = egui::pos2(target_rect.left() + w, target_rect.top());
                let rect = egui::Rect::from_points(&[target_rect.left_bottom(), handle_pos]);
                (rect, handle_pos)
            } else {
                let handle_pos = egui::pos2(target_rect.right() - w, target_rect.top());
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
                [rect.left_bottom(), rect.right_top()]
            } else {
                [rect.left_top(), rect.right_bottom()]
            };

            painter.line_segment(points, egui::Stroke::new(1.0, c));
            painter.rect_filled(handle_area, 1.0, egui::Color32::DARK_GRAY);

            ui_handle.on_hover_cursor(egui::CursorIcon::PointingHand)
        };
        let range = (self.range.end() - self.range.start()) as f32 / gui::SAMPLES_PER_PIXEL_DEFAULT;
        let scale = move |sec| sec * 44100.0 / gui::SAMPLES_PER_PIXEL_DEFAULT;
        let descale = move |pix| pix * gui::SAMPLES_PER_PIXEL_DEFAULT / 44100.0;
        let in_width = scale(self.param.time_in.load());
        let out_width = scale(self.param.time_out.load());
        let start = make_circle(in_width, true);
        if start.drag_started() {
            self.start_tmp = in_width;
        }
        if start.dragged() {
            self.start_tmp += start.drag_delta().x;
            let v = (self.start_tmp + start.drag_delta().x).clamp(0.0, range - out_width);
            self.param.time_in.store(descale(v));
        }
        if start.drag_released() {
            self.start_tmp = 0.0;
        }
        let end = make_circle(out_width, false);
        if end.drag_started() {
            self.end_tmp = out_width;
        }
        if end.dragged() {
            //accumlate delta.
            self.end_tmp -= end.drag_delta().x;
            let v = (self.end_tmp - end.drag_delta().x).clamp(0.0, range - in_width);

            self.param.time_out.store(descale(v));
        }
        if end.drag_released() {
            self.end_tmp = 0.0;
        }
        start.union(end)
    }
}
