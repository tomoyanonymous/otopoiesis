use super::*;
use crate::parameter::{FloatParameter, Parameter};
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

//各モデルは初期化時にArc<atomicを含む型>を受け取り状態を共有する
//共有が不要な内部パラメーターは普通のfloatなどでOK
pub struct SharedParams {
    pub amp: FloatParameter,
    pub freq: FloatParameter,
}
pub struct SineModel {
    phase: f32,
    params: Arc<SharedParams>,
}

impl SineModel {
    pub fn new(params: Arc<SharedParams>) -> Self {
        Self { phase: 0.0, params }
    }
    pub fn render_sample_internal(&mut self, out: &mut f32, info: &PlaybackInfo) {
        let twopi = std::f32::consts::PI * 2.;
        self.phase =
            (self.phase + twopi * self.params.freq.get() / info.sample_rate as f32) % twopi;
        *out = self.phase.sin() * self.params.amp.get();
    }
}

impl GeneratorComponent for SineModel {
    type Params = SharedParams;
    fn get_params(&self) -> &Self::Params {
        self.params.as_ref()
    }
    fn render_sample(&mut self, out: &mut f32, info: &PlaybackInfo) {
        self.render_sample_internal(out, info)
    }
}
