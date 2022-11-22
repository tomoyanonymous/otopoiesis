use crate::gui::{Component, ComponentBase};
use crate::parameter::Parameter;
use crate::region;
use crate::*;
use std::sync::Arc;

// use nannou_egui::*;
use nannou_egui::egui;

pub struct Model {
    samples: Vec<f32>,
    pub osc_params: Arc<oscillator::SharedParams>,
    pub region_params: Arc<region::Params>,
    on_start_hover: bool,
    on_end_hover: bool,
    horizontal_scale: f32,
    amp_tmp: f32,
    freq_tmp: f32,
    base: ComponentBase,
}

fn region_bar(ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
    let (response, mut painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = response.rect;

    let pos = [
        egui::pos2(response.rect.center().x, response.rect.top()),
        egui::pos2(response.rect.center().x, response.rect.bottom()),
    ];
    if response.hovered() {
        painter.rect_filled(response.rect, 0., ui.style().visuals.window_shadow.color);
    }
    if response.dragged() {
        let mut bold = egui::Stroke::default();
        bold.width = 1.0;
        bold.color = ui.style().visuals.strong_text_color();
        painter.line_segment(pos, bold);
    }
    if let Some(cursor_pos) = response.interact_pointer_pos() {
        let test_text = if response.rect.contains(cursor_pos) {
            format!("{:?}", cursor_pos)
        } else {
            "not_hovered".to_string()
        };
        painter.debug_text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            Color32::BLACK,
            test_text,
        );
    }
    painter.debug_rect(response.rect, Color32::RED, "bar");
    response
}

fn default_graph(
    label: impl std::hash::Hash,
    iter: impl Iterator<Item = egui::plot::Value>,
) -> egui::plot::Plot {
    let line = egui::plot::Line::new(egui::plot::Values::from_values_iter(iter));
    egui::plot::Plot::new(label)
        .line(line)
        .allow_drag(false)
        .allow_zoom(false)
}

impl egui::Widget for &mut Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let scaling_factor = 50;

        let bar_size = egui::vec2(10.0, 100.0);
        let x_size = (self.region_params.range.getrange() / scaling_factor) as f32;
        let region_size = egui::vec2(x_size, 100.0);
        let max_rect = egui::Rect::from_x_y_ranges(
            0.0..=(self.region_params.range.start() / scaling_factor) as f32,
            0.0..=100.0,
        )
        .translate(egui::vec2(ui.min_rect().left(), 0.));
        let response = ui
            .allocate_ui_at_rect(max_rect, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0., 0.);
                    //draw left handle
                    ui.add_space((self.region_params.range.start() / scaling_factor) as f32);
                    let left = region_bar(ui, bar_size);

                    if let Some(cursor_pos) = left.interact_pointer_pos() {
                        let new_start = (cursor_pos.x as u64 * scaling_factor)
                            .min(self.region_params.range.end());
                        self.region_params.range.set_start(new_start);
                    }
                    //draw main plot

                    let center = ui.add_sized(
                        region_size,
                        default_graph(
                            "region",
                            self.samples.iter().enumerate().map(|(i, s)| {
                                let x = nannou::math::map_range(
                                    i as f32,
                                    0.,
                                    self.samples.len() as f32,
                                    0.,
                                    x_size,
                                );
                                let y = *s * 100.0 * self.get_current_amp();
                                egui::plot::Value::new(x, y)
                            }),
                        ),
                    );
                    //draw right handle
                    let right = region_bar(ui, bar_size);
                    if let Some(cursor_pos) = right.interact_pointer_pos() {
                        let new_size = ((cursor_pos.x - center.rect.left() - right.rect.width())
                            as u64
                            * scaling_factor)
                            .min(self.region_params.max_size);
                        self.region_params
                            .range
                            .set_end(self.region_params.range.start() + new_size);
                    }
                    //debug
                    ui.add_sized(
                        egui::vec2(20.0, 100.),
                        egui::Label::new(format!("{:?}", x_size)),
                    );
                })
                .response
            })
            .response;
        response.ctx.debug_painter().rect(
            response.rect,
            0.,
            Color32::TRANSPARENT,
            egui::Stroke::new(1., Color32::RED),
        );

        response
    }
}

pub fn range_to_bound(horizontal_scale: f32, range: (u64, u64)) -> Rect {
    Rect::from_x_y_w_h(
        range.0 as f32 * horizontal_scale,
        50.,
        range.1 as f32 * horizontal_scale,
        400.,
    )
}

impl Model {
    pub fn update_samples(&mut self) {
        let mut phase = 0.0f32;
        let len = self.samples.len();
        for s in self.samples.iter_mut() {
            *s = phase.sin();
            let twopi = std::f32::consts::PI * 2.0;
            //とりあえず、440Hzで1周期分ということで
            let ratio = self.osc_params.freq.get() / 440.0;
            let increment = ratio * twopi / len as f32;
            phase = (phase + increment) % twopi;
        }
    }

    pub fn new(
        bound: Rect,
        osc_params: Arc<oscillator::SharedParams>,
        region_params: Arc<region::Params>,
    ) -> Self {
        let size = 512;
        let samples = vec![0f32; size];
        let horizontal_scale = 0.01;
        let amp_tmp = osc_params.amp.get();
        let freq_tmp = osc_params.freq.get();
        let rect = range_to_bound(horizontal_scale, region_params.range.get_pair());
        let base = ComponentBase::new(rect);
        let mut res = Self {
            samples,
            osc_params,
            region_params,
            horizontal_scale,
            on_end_hover: false,
            on_start_hover: false,
            amp_tmp,
            freq_tmp,
            base,
        };
        res.update_samples();
        res
    }
    pub fn get_current_amp(&self) -> f32 {
        self.osc_params.amp.get().abs()
    }
}
