use crate::data;
use crate::parameter::Parameter;
use egui::Color32;
use std::sync::{atomic::Ordering, Arc};
use nannou_egui::egui;

pub struct Model {
    pub params: Arc<data::Region>,
    pub label: String,
    samples: Vec<f32>,
    // pub osc_params: Arc<oscillator::SharedParams>,
}
impl std::hash::Hash for Model {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state)
    }
}

struct UiBar;
impl egui::Widget for &UiBar {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        if response.hovered() {
            painter.rect_filled(rect, 0., ui.style().visuals.weak_text_color());
        }

        if response.dragged() {
            painter.rect_filled(rect, 0., ui.style().visuals.strong_text_color());
        }
        response
    }
}

impl egui::Widget for Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let scaling_factor = super::SAMPLES_PER_PIXEL_DEFAULT;
        let x_start = self.params.range.start() as f32 / scaling_factor;
        let x_size = self.params.range.getrange() as f32 / scaling_factor;
        let y_size = 100.0;
        let region_size = egui::vec2(x_size, y_size);
        // .translate(egui::vec2(ui.min_rect().left(), 0.));
        let response = ui
            .vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0., 0.);
                    //draw left handle
                    ui.add_space(x_start);
                    let bar_width = 10.;
                    // let mut lrect = egui::Rect::from(graph.rect);
                    // lrect.set_right(lrect.left() + bar_width);
                    let size = egui::vec2(bar_width, y_size);
                    let left = ui.add_sized(size, &UiBar);
                    let maxsize = self.params.max_size.load(Ordering::Relaxed);
                    let start = self.params.range.start();
                    let end = self.params.range.end();
                    if let Some(cursor_pos) = left.interact_pointer_pos() {
                        let new_start = (((cursor_pos.x - bar_width) * scaling_factor) as u64)
                            .min(end)
                            .max(end.max(maxsize) - maxsize);
                        self.params.range.set_start(new_start);
                    }
                    let iter = self.samples.iter().enumerate().map(|(i, s)| {
                        let x = nannou::math::map_range(
                            i as f32,
                            0.,
                            self.samples.len() as f32,
                            0.,
                            x_size,
                        );
                        let y = *s * y_size * self.get_current_amp();
                        egui::plot::Value::new(x, y)
                    });
                    let line = egui::plot::Line::new(egui::plot::Values::from_values_iter(iter));
                    let graph_component = egui::plot::Plot::new(self.params.label.clone())
                        .line(line)
                        .allow_drag(false)
                        .allow_zoom(false)
                        .show_x(false)
                        .show_y(false)
                        .show_axes([false, true])
                        .min_size(egui::vec2(0., 0.));
                    let graph = ui
                        .add_sized(region_size, graph_component)
                        .interact(egui::Sense::hover());

                    // ui.painter_at(lrect)

                    let mut rrect = egui::Rect::from(graph.rect);
                    rrect.set_left(rrect.right() - 10.);
                    let right = ui.add_sized(size, &UiBar);

                    if let Some(cursor_pos) = right.interact_pointer_pos() {
                        let new_end = ((cursor_pos.x - bar_width) * scaling_factor) as u64;

                        let new_size = (new_end.max(start) - start).min(maxsize);
                        self.params.range.set_end(start + new_size);
                    }
                });

                {
                    let data::Generator::Oscillator(osc) = self.params.generator.as_ref();

                    let range = &osc.freq.range;
                    ui.add(
                        egui::Slider::from_get_set(
                            *range.start() as f64..=*range.end() as f64,
                            |v: Option<f64>| {
                                if let Some(n) = v {
                                    osc.freq.set(n as f32)
                                }
                                osc.freq.get() as f64
                            },
                        )
                        .logarithmic(true),
                    );
                };
            })
            .response;

        // let debugger = response.ctx.debug_painter();
        // debugger.rect(
        //     response.rect,
        //     0.,
        //     Color32::TRANSPARENT,
        //     egui::Stroke::new(1., Color32::RED),
        // );

        response
    }
}

pub fn range_to_bound(horizontal_scale: f32, range: (u64, u64)) -> nannou::geom::Rect {
    nannou::geom::Rect::from_x_y_w_h(
        range.0 as f32 * horizontal_scale,
        50.,
        range.1 as f32 * horizontal_scale,
        400.,
    )
}

impl Model {
    pub fn update_samples(&mut self) {
        // let mut phase = 0.0f32;
        let len = self.samples.len();
        let gen = &self.params.generator;
        match gen.as_ref() {
            data::Generator::Oscillator(osc) => {
                let mut phase_gui = 0.0f32;
                for s in self.samples.iter_mut() {
                    *s = phase_gui.sin() * osc.amp.get();
                    let twopi = std::f32::consts::PI * 2.0;
                    //とりあえず、440Hzで1周期分ということで
                    let ratio = osc.freq.get() / 440.0;
                    let increment = ratio * twopi / len as f32;
                    phase_gui = (phase_gui + increment) % twopi;
                }
            }
        }
    }

    pub fn new(params: Arc<data::Region>, labeltext: impl ToString) -> Self {
        let size = 512;
        let samples = vec![0f32; size];
        let label = labeltext.to_string();
        let mut res = Self {
            samples,
            label,
            params,
        };
        res.update_samples();
        res
    }
    pub fn get_current_amp(&self) -> f32 {
        // self.osc_params.amp.get().abs()
        1.0
    }
}
