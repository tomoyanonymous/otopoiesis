use crate::{audio, data, parameter::Parameter};
/// 一時的にaudio Componentを生成して画像用の波形をNon-Realtime Renderする

pub struct State {
    samples: Vec<f32>,
}

impl State {
    pub fn new(size: usize) -> Self {
        Self {
            samples: vec![0.0; size],
        }
    }
}

pub struct Generator<'a> {
    param: &'a mut data::Generator,
    state: &'a mut State,
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
            *o = is.chunks(chs).map(|i| i[0].abs()).reduce(f32::max).unwrap();
        });
}

impl<'a> Generator<'a> {
    pub fn new(param: &'a mut data::Generator, state: &'a mut State) -> Self {
        Self { param, state }
    }
    pub fn update_samples(&mut self, width: f32) {
        // let mut phase = 0.0f32;
        let sample_rate = 44100u32;
        let channels = 2;
        let len_samples =
            (sample_rate as f32 * width / super::PIXELS_PER_SEC_DEFAULT).ceil() as usize;
        let pix_len = width.ceil() as usize;
        let dummy = vec![0.0f32; 0];
        let mut buf = vec![0.0f32; len_samples * channels];

        let mut audio_component = audio::generator::get_component_for_generator(self.param);
        audio_component.render(
            &dummy,
            &mut buf,
            &audio::PlaybackInfo {
                sample_rate,
                current_time: 0,
                frame_per_buffer: pix_len as u64,
                channels: 2,
            },
        );
        self.state.samples.resize(pix_len, 0.0f32);
        reduce_samples(&buf, &mut self.state.samples);
    }
    fn make_graph_sized(&mut self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let points = self
            .state
            .samples
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let x = egui::emath::remap(
                    i as f64,
                    0.0..=self.state.samples.len() as f64,
                    0.0..=size.x as f64,
                );
                let y = *s * size.y;
                [x, y as f64]
            })
            .collect::<Vec<_>>();

        let plot = egui::plot::Plot::new(ui.auto_id_with("generator"))
            .allow_drag(false)
            .allow_zoom(false)
            .allow_boxed_zoom(false)
            .allow_scroll(false)
            .allow_double_click_reset(false)
            .width(size.x)
            .height(size.y)
            .show_x(false)
            .show_y(false)
            .show_axes([false, true])
            .min_size(egui::vec2(0., 0.))
            .set_margin_fraction(egui::vec2(0., 0.));

        plot.show(ui, |plot_ui| {
            plot_ui.line(egui::plot::Line::new(points).fill(0.0))
        })
        .response
        .on_hover_cursor(egui::CursorIcon::Grab)
        .interact(egui::Sense::click_and_drag())
    }
}

impl<'a> egui::Widget for Generator<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            let res = self.make_graph_sized(ui, ui.available_size());
            if self.state.samples.is_empty() {
                self.update_samples(res.rect.width());
            }
            let _controller = match &self.param {
                data::Generator::Oscillator(_kind, osc) => {
                    let range = &osc.freq.range;
                    let slider = ui.add(
                        egui::Slider::from_get_set(
                            *range.start() as f64..=*range.end() as f64,
                            |v: Option<f64>| {
                                if let Some(n) = v {
                                    osc.freq.set(n as f32);
                                }
                                osc.freq.get() as f64
                            },
                        )
                        .logarithmic(true),
                    );
                    if slider.drag_released() && slider.changed() {
                        self.update_samples(res.rect.width());
                    }
                    slider
                }
                data::Generator::Noise() => todo!(),
                data::Generator::Constant => unimplemented!(),
                #[cfg(not(feature = "web"))]
                data::Generator::FilePlayer(param) => ui.label(param.path.to_string()),
            };
            res
        })
        .inner
    }
}
