use crate::{
    audio::{
        self,
        region::{RangedComponent, RangedComponentDyn},
    },
    data,
    gui::parameter::slider_from_parameter,
    utils::AtomicRange,
};
use egui::{epaint::Shape, Pos2, Sense, Vec2};

use std::ops::RangeInclusive;

pub struct State {
    samples: Vec<f32>,
    shape: Shape,
}

impl State {
    pub fn new() -> Self {
        Self {
            samples: vec![0.0; 0],
            shape: Shape::Noop,
        }
    }
}
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// get peak samples
fn reduce_samples(input: &[f32], output: &mut [f32]) {
    let rate = super::PIXELS_PER_SEC_DEFAULT;
    output
        .iter_mut()
        .zip(input.chunks(rate as usize))
        .for_each(|(o, is)| {
            //take left channel
            let chs = 2;
            *o = is.chunks(chs).map(|i| i[0]).last().unwrap();
        });
}
pub trait GeneratorUI<'a> {
    fn get_generator(&self) -> &data::Generator;
    fn get_samples(&mut self) -> &mut Vec<f32>;
    fn get_displayed_range(&self) -> RangeInclusive<f64>;

    fn get_displayed_duration(&self) -> f64 {
        let range = self.get_displayed_range();
        range.end() - range.start()
    }
    fn update_samples(&mut self) {
        let width = self.get_displayed_duration() * super::PIXELS_PER_SEC_DEFAULT as f64;
        let pix_len = width.ceil() as usize;
        let sample_rate = 44100u32;
        let channels = 2;
        let numsamples = (sample_rate as f64 * self.get_displayed_duration()).ceil() as usize;
        let mut buf = vec![0.0f32; numsamples * channels];
        let audio_component = audio::generator::get_component_for_generator(self.get_generator());
        let mut ranged_component = RangedComponentDyn::new(
            audio_component,
            AtomicRange::from(self.get_displayed_range()),
        );
        ranged_component.render_offline(&mut buf, sample_rate, channels as u64);
        self.get_samples().resize(pix_len, 0.0f32);
        reduce_samples(&buf, self.get_samples());
    }
}

pub struct Generator<'a> {
    param: &'a data::Generator,
    displayed_range: &'a AtomicRange<f64>,
    state: &'a mut State,
}

impl<'a> GeneratorUI<'a> for Generator<'a> {
    fn get_generator(&self) -> &data::Generator {
        self.param
    }

    fn get_samples(&mut self) -> &mut Vec<f32> {
        &mut self.state.samples
    }

    fn get_displayed_range(&self) -> RangeInclusive<f64> {
        self.displayed_range.start()..=self.displayed_range.end()
    }
}

impl<'a> Generator<'a> {
    pub fn new(
        param: &'a data::Generator,
        displayed_range: &'a AtomicRange<f64>,
        state: &'a mut State,
    ) -> Self {
        Self {
            param,
            displayed_range,
            state,
        }
    }
    fn get_size(&self) -> Vec2 {
        let width = self.get_displayed_duration() as f32 * super::PIXELS_PER_SEC_DEFAULT;
        let height = super::TRACK_HEIGHT;
        egui::vec2(width, height)
    }
    pub fn update_shape(&mut self, style: &egui::Style) {
        self.update_samples();

        let from = 0.0..=self.get_samples().len() as f64;
        let to = 0.0..=self.get_size().x as f64;
        let y_origin = self.get_size().y;

        let points_to_draw = self
            .get_samples()
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let x = egui::emath::remap(i as f64, from.clone(), to.clone());
                let y = *s * y_origin * 0.5;
                egui::pos2(x as f32, y)
            })
            .collect::<Vec<Pos2>>();
        let mut visu = style.visuals.widgets.active.clone();
        visu.fg_stroke.width = 1.0;
        self.state.shape = Shape::line(points_to_draw, visu.fg_stroke);
    }
}

impl<'a> egui::Widget for Generator<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            let (response, painter) = ui.allocate_painter(self.get_size(), Sense::click_and_drag());

            if self.state.samples.is_empty() {
                self.update_shape(&ui.ctx().style());
            }
            let mut shape_c = self.state.shape.clone();
            shape_c.translate(egui::vec2(
                response.rect.left_top().x,
                response.rect.size().y,
            ));
            painter.add(shape_c);
            ui.set_max_width(self.state.shape.visual_bounding_rect().width());
            let _controller = ui.push_id(ui.next_auto_id(), |ui| {
                egui::menu::menu_button(ui, "parameter", |ui| {
                    //  ui.collapsing("parameter", |ui| {
                    match &self.param {
                        data::Generator::Oscillator(_kind, osc) => {
                            let response = ui
                                .vertical(|ui| {
                                    let f = slider_from_parameter(&osc.freq, true, ui);
                                    let a = slider_from_parameter(&osc.amp, false, ui);
                                    let p = slider_from_parameter(&osc.phase, false, ui);
                                    f.union(a.union(p))
                                })
                                .response;
                            if response.drag_released() && response.changed() {
                                self.update_shape(&ui.ctx().style());
                            }
                            response
                        }
                        data::Generator::Noise() => ui.label("Noise"),
                        data::Generator::Constant(param) => {
                            let slider = slider_from_parameter(&param, true, ui);
                            if slider.drag_released() && slider.changed() {
                                self.update_shape(&ui.ctx().style());
                            }
                            slider
                        }
                        #[cfg(not(feature = "web"))]
                        data::Generator::FilePlayer(param) => ui.label(param.path.to_string()),
                    };
                });
            });
            response
                .on_hover_cursor(egui::CursorIcon::Grab)
                .interact(egui::Sense::click_and_drag())
        })
        .inner
    }
}
