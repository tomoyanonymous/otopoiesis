use super::*;

pub trait GeneratorComponent {
    fn render_sample(&mut self, out: &mut f32);
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
    fn render(&mut self, _input: &[f32], output: &mut [f32]) {
        for oframe in output.chunks_mut(2) {
            let mut ch = 0;
            for s in oframe {
                if ch == 0 {
                    self.render_sample(s)
                } else {
                    *s = 0.0;
                }
                ch += 1;
            }
        }
    }
}

pub struct OscillatorModel {
    phase: f32,
    pub amp: f32,
    pub freq: f32,
    pub sr: f32,
}
impl OscillatorModel {
    pub fn new(amp: f32, freq: f32, sr: f32) -> Self {
        Self {
            phase: 0.0,
            amp,
            freq,
            sr,
        }
    }
}

impl GeneratorComponent for OscillatorModel {
    fn render_sample(&mut self, out: &mut f32) {
        let twopi = std::f32::consts::PI * 2.;
        self.phase = (self.phase + twopi * self.freq / self.sr) % twopi;
        *out = self.phase.sin() * self.amp;
    }
}
