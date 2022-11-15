use super::*;
use crate::parameter;

pub trait GeneratorComponent {
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
        1
    }
    fn render(&mut self, _input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        for oframe in output.chunks_mut(2) {
            let mut ch = 0;
            for s in oframe {
                if ch == 0 {
                    self.render_sample(s, info)
                } else {
                    *s = 0.0;
                }
                ch += 1;
            }
        }
    }
}

//各モデルは初期化時にArc<atomicを含む型>を受け取り状態を共有する
//共有が不要な内部パラメーターは普通のfloatなどでOK
pub struct SineWave {
    phase: f32,
    pub amp: Arc<parameter::FloatParameter>,
    pub freq: Arc<parameter::FloatParameter>,
}

impl SineWave {
    pub fn new(amp: Arc<parameter::FloatParameter>, freq: Arc<parameter::FloatParameter>) -> Self {
        Self {
            phase: 0.0,
            amp,
            freq,
        }
    }
}

impl GeneratorComponent for SineWave {
    fn render_sample(&mut self, out: &mut f32, info: &PlaybackInfo) {
        let twopi = std::f32::consts::PI * 2.;
        self.phase = (self.phase + twopi * self.freq.get() / info.sample_rate) % twopi;
        *out = self.phase.sin() * self.amp.get();
    }
}
