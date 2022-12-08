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
    T: GeneratorComponent + Clone + std::fmt::Debug,
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

#[derive(Clone, Debug)]
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

#[derive(Debug)]
pub struct FadeModel {
    pub param: Arc<data::FadeParam>,
    pub origin: super::region::Model,
    buffer: Vec<f32>,
}

impl FadeModel {
    pub fn new(transformer: &data::RegionTransformer) -> Self {
        let param = match transformer.filter.as_ref() {
            data::RegionFilter::Gain => todo!(),
            data::RegionFilter::FadeInOut(p) => p,
        };
        Self {
            param: param.clone(),
            origin: super::region::Model::new(transformer.origin.clone(), 2),
            buffer: vec![],
        }
    }
    fn render_offline(&mut self, info: &PlaybackInfo) {
        self.buffer
            .resize(self.origin.interleaved_samples_cache.len(), 0.0);
        self.origin.render_offline(info);
        let chs = self.get_output_channels() as usize;
        self.origin
            .interleaved_samples_cache
            .chunks(chs)
            .zip(self.buffer.chunks_mut(chs))
            .enumerate()
            .for_each(|(count, (v_per_channel, o_per_channel))| {
                let in_time = self.param.time_in.load() as f64 * info.sample_rate as f64;
                let out_time = self.param.time_out.load() as f64 * info.sample_rate as f64;
                let now = count as f64;

                let len = self.origin.params.range.getrange() as f64;
                let mut gain = 1.0;
                if (0.0..=in_time).contains(&now) {
                    gain = (now as f64 / in_time).clamp(0.0, 1.0);
                }

                if (len - out_time..=len).contains(&now) {
                    gain = ((len - now) as f64 / out_time).clamp(0.0, 1.0);
                }
                if now > len {
                    gain = 0.0;
                }

                v_per_channel
                    .iter()
                    .map(|s| s * gain as f32)
                    .zip(o_per_channel.iter_mut())
                    .for_each(|(v, o)| {
                        *o = v;
                    });
            });
    }
}

impl Component for FadeModel {
    fn get_input_channels(&self) -> u64 {
        0
    }
    fn get_output_channels(&self) -> u64 {
        2
    }
    fn prepare_play(&mut self, info: &PlaybackInfo) {
        self.render_offline(info);
    }
    fn render(&mut self, _input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        let range = &self.origin.params.range;
        let chs = info.channels as usize;
        for (count, out_per_channel) in output.chunks_mut(chs).enumerate() {
            let now = (info.current_time + count) as u64;
            let in_range = now < range.getrange();
            if in_range {
                let read_point = now as usize * chs;
                out_per_channel
                    .iter_mut()
                    .enumerate()
                    .for_each(|(ch, s)| *s = self.buffer[read_point + ch]);
            }
        }
    }
}

pub fn get_component_for_generator(kind: &data::Generator) -> Box<dyn Component + Send + Sync> {
    match kind {
        data::Generator::Oscillator(osc) => Box::new(SineModel::new(Arc::clone(osc))),
        data::Generator::Transformer(t) => {
            let t = FadeModel::new(t);
            Box::new(t)
        }
    }
}
