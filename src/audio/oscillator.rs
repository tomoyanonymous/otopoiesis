use super::*;
use crate::data;
use crate::parameter::Parameter;
use std::sync::Arc;
pub trait GeneratorComponent {
    type Params;
    fn get_params(&self) -> &Self::Params;
    fn reset_phase(&mut self);
    fn render_sample(&mut self, out: &mut f32, info: &PlaybackInfo);
}
impl<T> Component for T
where
    T: GeneratorComponent,
{
    fn get_input_channels(&self) -> u64 {
        0
    }
    fn get_output_channels(&self) -> u64 {
        2
    }

    fn prepare_play(&mut self, _info: &PlaybackInfo) {
        self.reset_phase();
    }
    fn render(&mut self, _input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        for (_count, out_per_channel) in output
            .chunks_mut(self.get_output_channels() as usize)
            .enumerate()
        {
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
    pub phase_internal: f32,
}

impl SineModel {
    pub fn new(params: Arc<data::OscillatorParam>) -> Self {
        Self {
            params: Arc::clone(&params),
            phase_internal: params.phase.get(),
        }
    }
    pub fn render_sample_internal(&mut self, out: &mut f32, info: &PlaybackInfo) {
        let twopi = std::f32::consts::PI * 2.;
        let params = &self.params;
        self.phase_internal =
            (self.phase_internal + twopi * params.freq.get() / info.sample_rate as f32) % twopi;
        *out = self.phase_internal.sin() * self.params.amp.get();
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
    fn reset_phase(&mut self) {
        self.phase_internal = self.get_params().phase.get()
    }
}

pub fn get_component_for_generator(kind: &data::Generator) -> Box<dyn Component + Send> {
    match kind {
        data::Generator::Oscillator(osc) => Box::new(SineModel::new(Arc::clone(osc))),
    }
}
