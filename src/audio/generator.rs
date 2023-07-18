use super::*;
use crate::data;
pub mod oscillator;
pub mod fileplayer;

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
pub struct Constant();
impl Component for Constant {
    fn get_input_channels(&self) -> u64 {
        0
    }
    fn get_output_channels(&self) -> u64 {
        2
    }

    fn prepare_play(&mut self, _info: &PlaybackInfo) {}
    fn render(&mut self, _input: &[f32], output: &mut [f32], _info: &PlaybackInfo) {
        output.fill(1.0);
    }
}

pub fn get_component_for_generator(kind: &data::Generator) -> Box<dyn Component + Send + Sync> {
    match kind {
        data::Generator::Oscillator(fun, param) => Box::new(match fun {
            data::OscillatorFun::SineWave => oscillator::sinewave(param.clone()),
            data::OscillatorFun::SawTooth(dir) => oscillator::saw(param.clone(), dir.clone()),
            data::OscillatorFun::Rectanglular(duty) => {
                oscillator::rect(param.clone(), duty.clone())
            }
            data::OscillatorFun::Triangular => oscillator::triangle(param.clone()),
        }),
        data::Generator::Constant => Box::new(Constant()),
        data::Generator::Noise() => todo!(),
    }
}
