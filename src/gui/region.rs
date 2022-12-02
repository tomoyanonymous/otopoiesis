use crate::data;
use crate::gui;
use crate::parameter::Parameter;
use egui;

use std::ops::RangeInclusive;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

enum HandleMode {
    Start,
    End,
}
struct UiBar {
    state: Arc<AtomicU64>,
    saved_state: u64,
    range: RangeInclusive<u64>,
    mode: HandleMode,
}
impl UiBar {
    pub fn new(state: Arc<AtomicU64>, mode: HandleMode) -> Self {
        let init = state.load(Ordering::Relaxed);
        Self {
            state,
            saved_state: init,
            range: 0..=init,
            mode,
        }
    }
    fn set_limit(&mut self, range: RangeInclusive<u64>) {
        self.range = range;
    }
    fn react(&mut self, response: &egui::Response) {
        if response.drag_started() {
            self.saved_state = self.state.load(Ordering::Relaxed);
        }
        if let Some(_pos) = response.interact_pointer_pos() {
            let new_v = (self.saved_state as i64
                + ((response.drag_delta().x * gui::SAMPLES_PER_PIXEL_DEFAULT) as i64))
                .max(0) as u64;

            self.state.store(
                new_v.clamp(*self.range.start(), *self.range.end()),
                Ordering::Relaxed,
            );
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
        let text = response.hover_pos().map_or("none".to_string(), |p| {
            format!("{:?}/offset:{}", p, rect_x).to_string()
        });
        response.on_hover_text_at_pointer(text)
        // response
    }
}

pub struct Model {
    pub params: Arc<data::Region>,
    pub label: String,
    samples: Vec<f32>,
    range_handles: [UiBar; 2], // pub osc_params: Arc<oscillator::SharedParams>,
    offset_saved: i64,
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
        let handle_left = UiBar::new(params.range.0.clone(),HandleMode::Start);
        let handle_right = UiBar::new(params.range.1.clone(),HandleMode::End);

        let mut res = Self {
            samples,
            label,
            params,
            range_handles: [handle_left, handle_right],
            offset_saved: 0,
        };
        res.update_samples();
        res
    }
    pub fn get_current_amp(&self) -> f32 {
        // self.osc_params.amp.get().abs()
        1.0
    }
}

impl std::hash::Hash for Model {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state)
    }
}

impl egui::Widget for &mut Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let scaling_factor = super::SAMPLES_PER_PIXEL_DEFAULT;
        let bar_width = 5.;

        let width = self.params.range.getrange() as f32 / scaling_factor;
        let start = self.params.range.start();
        let end = self.params.range.end();
        let maxsize = self.params.max_size.load(Ordering::Relaxed);
        let min_start = (end as i64 - maxsize as i64).max(0) as u64;
        let max_end = start + maxsize;
        self.range_handles[0].set_limit(min_start..=end);

        self.range_handles[1].set_limit(start..=max_end);

        let height = 100.0;
        let response = ui
            .vertical(|ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0., 0.);
                    //draw left handle

                    let size = egui::vec2(bar_width, height);
                    ui.add_sized(size, &mut self.range_handles[0]);
                    let points = self
                        .samples
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            let x = egui::emath::remap(
                                i as f64,
                                0.0..=self.samples.len() as f64,
                                0.0..=width as f64,
                            );

                            let y = *s * height * self.get_current_amp();
                            [x, y as f64]
                        })
                        .collect::<Vec<_>>();

                    let mut graph = egui::plot::Plot::new(self.params.label.clone())
                        .allow_drag(false)
                        .allow_zoom(false)
                        .allow_boxed_zoom(false)
                        .allow_scroll(false)
                        .allow_double_click_reset(false)
                        .width(width)
                        .height(height)
                        .show_x(false)
                        .show_y(false)
                        .show_axes([false, true])
                        .min_size(egui::vec2(0., 0.))
                        .show(ui, |plot_ui| plot_ui.line(egui::plot::Line::new(points)))
                        .response
                        .on_hover_cursor(egui::CursorIcon::Grab)
                        .interact(egui::Sense::click_and_drag());
                    if graph.drag_started() {
                        self.offset_saved = self.params.range.start() as i64;
                    }
                    if graph.dragged() {
                        let offset = (graph.drag_delta().x * gui::SAMPLES_PER_PIXEL_DEFAULT) as i64;
                        self.params.range.shift(offset);
                        graph = graph.on_hover_cursor(egui::CursorIcon::Grabbing)
                    }
                    if graph.drag_released() {
                        self.offset_saved = 0;
                    }
                    ui.add_sized(size, &mut self.range_handles[1]);

                    // ui.painter_at(lrect)

                    // let mut rrect = egui::Rect::from(graph.rect);
                    // rrect.set_left(rrect.right() - 10.);
                    // let right = ui.add_sized(size, &UiBar);

                    // if let Some(cursor_pos) = right.interact_pointer_pos() {
                    //     let new_end = ((cursor_pos.x - bar_width) * scaling_factor) as u64;

                    //     let new_size = (new_end.max(start) - start).min(maxsize);
                    //     self.params.range.set_end(start + new_size);
                    // }
                });

                {
                    let data::Generator::Oscillator(osc) = self.params.generator.as_ref();

                    let range = &osc.freq.range;
                    ui.horizontal(|ui| {
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
                    })
                };
            })
            .response;

        response
    }
}
