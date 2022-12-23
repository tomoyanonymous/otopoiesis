use crate::{data, parameter::Parameter};

pub struct Generator {
    param: data::Generator,
    samples: Vec<f32>,
}

impl Generator {
    pub fn new(param: data::Generator) -> Self {
        let size = 512;
        let mut res = Self {
            param,
            samples: vec![0.0; size],
        };
        res.update_samples();
        res
    }
    pub fn update_samples(&mut self) {
        // let mut phase = 0.0f32;
        let len = self.samples.len();

        match &self.param {
            data::Generator::Oscillator(_kind, osc) => {
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
            data::Generator::Noise() => {
                unimplemented!()
            }
        }
    }
    fn make_graph_sized(&mut self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let width = size.x;
        let height = size.y;
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
                let y = *s * height;
                [x, y as f64]
            })
            .collect::<Vec<_>>();

        egui::plot::Plot::new(ui.auto_id_with("generator"))
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
            .set_margin_fraction(egui::vec2(0., 0.))
            .show(ui, |plot_ui| plot_ui.line(egui::plot::Line::new(points)))
            .response
            .on_hover_cursor(egui::CursorIcon::Grab)
            .interact(egui::Sense::click_and_drag())
    }
}

impl egui::Widget for &mut Generator {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            let res = self.make_graph_sized(ui, ui.available_size());
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
                    if slider.changed() {
                        self.update_samples();
                    }
                    slider
                }
                data::Generator::Noise() => todo!(),
            };
            res
        })
        .inner
    }
}
