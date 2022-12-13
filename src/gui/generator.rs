use crate::{data, gui, utils::AtomicRange};

use egui;
use std::{ops::RangeInclusive, sync::Arc};
pub struct FadeHandle {
    param: Arc<data::FadeParam>,
    pub range: RangeInclusive<u64>,
    start_tmp: f32,
    end_tmp: f32,
}
impl FadeHandle {
    pub fn new(param: Arc<data::FadeParam>, range: &AtomicRange) -> Self {
        Self {
            param: param,
            range: range.start()..=range.end(),
            start_tmp: 0.0,
            end_tmp: 0.0,
        }
    }
}

impl egui::Widget for &mut FadeHandle {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (target_rect, _response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::focusable_noninteractive());
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

pub struct TransformerModel {
    pub filter: FadeHandle,
    pub origin: Box<super::region::Model>,
}

impl TransformerModel {
    pub fn from(region: Arc<data::Region>) -> Self {
        let range = region.range.clone();
            match &region.content {
            data::Content::Generator(_) | data::Content::AudioFile(_) => todo!(),
            data::Content::Transformer(filter, origin) => {
                let filter = match filter.as_ref() {
                    data::RegionFilter::Gain => todo!(),
                    data::RegionFilter::Reverse => todo!(),
                    // do not use "origin.range" at here
                    data::RegionFilter::FadeInOut(p) => FadeHandle::new(p.clone(), &range),
                };
                Self {
                    filter: filter,
                    origin: Box::new(super::region::Model::new(Arc::clone(&origin), "filter1")),
                }
            }
        }
    }
}

impl egui::Widget for &mut TransformerModel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut wave = ui.add(self.origin.as_mut());
        // self.origin.params.range.0.store(self.filter.range.0.load());
        // self.origin.params.range.1.store(self.filter.range.1.load());

        self.filter.range = self.origin.params.range.0.load()..=self.origin.params.range.1.load();

        wave.rect
            .set_bottom(wave.rect.bottom().min(wave.rect.top() + 100.0));
        wave.rect = wave.rect.shrink2(egui::vec2(10.0, 0.0));
        ui.put(wave.rect, &mut self.filter)
    }
}
