use super::*;
use crate::data;
use crate::parameter::Parameter;
use std::sync::Arc;
pub trait GeneratorComponent {
    type Params;
    fn get_params(&self) -> &Self::Params;
    fn render_sample(&mut self, out: &mut f32, info: &PlaybackInfo);
}
impl<T> Component for T
where
    T: GeneratorComponent,
{
    fn get_input_channels(&self) -> usize {
        0
    }
    fn get_output_channels(&self) -> usize {
        2
    }
    fn prepare_play(&mut self, _info: &PlaybackInfo) {}
    fn render(&mut self, _input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        for (_count, out_per_channel) in output.chunks_mut(self.get_output_channels()).enumerate() {
            let mut res = 0.0;
            self.render_sample(&mut res, info);
            for (ch, s) in out_per_channel.iter_mut().enumerate() {
                if ch == 0 {
                    *s = res
                } else {
                    *s = 0.0
                }
            }
        }
    }
}
pub struct SineModel {
    pub params: Arc<data::OscillatorParam>,
}

impl SineModel {
    pub fn new(params: Arc<data::OscillatorParam>) -> Self {
        Self {
            params: Arc::clone(&params),
        }
    }
    pub fn render_sample_internal(&mut self, out: &mut f32, info: &PlaybackInfo) {
        let twopi = std::f32::consts::PI * 2.;
        let params = &self.params;
        params.phase.set(
            (params.phase.get() + twopi * params.freq.get() / info.sample_rate as f32) % twopi,
        );
        *out = params.phase.get().sin() * self.params.amp.get();
    }
}

impl GeneratorComponent for SineModel {
    type Params = data::OscillatorParam;
    fn get_params(&self) -> &Self::Params {
        self.params.as_ref()
    }
    fn render_sample(&mut self, out: &mut f32, info: &PlaybackInfo) {
        self.render_sample_internal(out, info)
    }
}
