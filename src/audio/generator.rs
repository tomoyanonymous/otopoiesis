use std::sync::Arc;

use super::*;
use crate::{
    data::{FilePlayerParam, OscillatorParam},
    parameter::{FloatParameter, Parameter, RangedNumeric, UIntParameter},
    script::{self, Expr, Value},
};
pub mod constant;
#[cfg(not(target_arch = "wasm32"))]
pub mod fileplayer;
pub mod noise;
pub mod oscillator;
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

// pub fn get_component_for_generator(kind: &data::Generator) -> Box<dyn Component + Send + Sync> {
//     match kind {
//         data::Generator::Oscillator(fun, param) => Box::new(match fun {
//             data::OscillatorFun::SineWave => oscillator::sinewave(*param.as_ref()),
//             data::OscillatorFun::SawTooth(dir) => oscillator::saw(*param.as_ref(), dir.clone()),
//             data::OscillatorFun::Rectanglular(duty) => {
//                 oscillator::rect(*param.as_ref(), duty.clone())
//             }
//             data::OscillatorFun::Triangular => oscillator::triangle(param.clone().as_ref().clone()),
//         }),
//         data::Generator::Constant(param) => Box::new(Constant(param.clone())),
//         data::Generator::Noise() => Box::new(Noise {}),
//         #[cfg(not(target_arch = "wasm32"))]
//         data::Generator::FilePlayer(param) => Box::new(fileplayer::FilePlayer::new(param.clone())),
//     }
// }
pub fn get_component_for_value(v: &script::Value) -> Box<dyn Component + Send + Sync> {
    match v {
        Value::Closure(_ids, _env,box Expr::App(box Expr::Literal(Value::ExtFunction(fname)), args)) => {
            match (fname.as_str(), &args.as_slice()) {
                (
                    "sinewave",
                    &[Expr::Literal(Value::Parameter(freq)), Expr::Literal(Value::Parameter(amp)), Expr::Literal(Value::Parameter(phase))],
                ) => Box::new(oscillator::sinewave(OscillatorParam {
                    amp: amp.clone(),
                    freq: freq.clone(),
                    phase: phase.clone(),
                })),
                (
                    "sawtooth",
                    &[Expr::Literal(Value::Parameter(freq)), Expr::Literal(Value::Parameter(amp)), Expr::Literal(Value::Parameter(phase)), Expr::Literal(Value::Parameter(dir))],
                ) => Box::new(oscillator::saw(
                    OscillatorParam {
                        amp: amp.clone(),
                        freq: freq.clone(),
                        phase: phase.clone(),
                    },
                    dir.clone(),
                )),
                (
                    "rectangular",
                    &[Expr::Literal(Value::Parameter(freq)), Expr::Literal(Value::Parameter(amp)), Expr::Literal(Value::Parameter(phase)), Expr::Literal(Value::Parameter(duty))],
                ) => Box::new(oscillator::rect(
                    OscillatorParam {
                        amp: amp.clone(),
                        freq: freq.clone(),
                        phase: phase.clone(),
                    },
                    duty.clone(),
                )),
                (
                    "triangular",
                    &[Expr::Literal(Value::Parameter(freq)), Expr::Literal(Value::Parameter(amp)), Expr::Literal(Value::Parameter(phase))],
                ) => Box::new(oscillator::sinewave(OscillatorParam {
                    amp: amp.clone(),
                    freq: freq.clone(),
                    phase: phase.clone(),
                })),
                ("constant", &[Expr::Literal(Value::Parameter(val))]) => {
                    Box::new(constant::Constant(val.clone()))
                }
                #[cfg(not(target_arch = "wasm32"))]
                ("fileplayer", &[Expr::Literal(Value::String(path))]) => {
                    let p = FilePlayerParam {
                        path: path.clone(),
                        channels: UIntParameter::new(2, "channels").set_range(0..=2),
                        start_sec: FloatParameter::new(0.0, "start").set_range(0.0..=10.0),
                        duration: FloatParameter::new(1.0, "duration").set_range(0.0..=10.0),
                    };
                    Box::new(fileplayer::FilePlayer::new(Arc::new(p)))
                }
                (_, _) => {
                    panic!("No matching generator")
                }
            }
        }
        _ => panic!("invalid components"),
    }
}
