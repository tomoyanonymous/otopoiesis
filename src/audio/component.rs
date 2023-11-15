use std::ops::RangeInclusive;

use crate::{utils::AtomicRange, script::{self, Value, EvalError}};

use super::PlaybackInfo;
pub trait Component: std::fmt::Debug {
    fn get_input_channels(&self) -> u64;
    fn get_output_channels(&self) -> u64;
    fn prepare_play(&mut self, info: &PlaybackInfo);
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo);
}


#[derive(Debug)]
pub struct ScriptComponent {
    val: Value,
}

impl ScriptComponent {
    pub fn try_new(val: &Value) -> Result<Self, EvalError> {
        let res = match val {
            Value::Closure(_ids, _env, box _body) => true,
            _ => false,
        };
        if res {
            Ok(Self { val: val.clone() })
        } else {
            Err(EvalError::TypeMismatch("not a closure".into()))
        }
    }
    fn compute_sample(&self, info: &PlaybackInfo) -> f64 {
        match &self.val {
            Value::Closure(_ids, env, box body) => {
                match body.eval(env.clone(), &Some(info), &mut None) {
                    Ok(Value::Number(res)) => res,
                    _ => 0.0,
                }
            }
            _ => 0.0,
        }
    }
}
impl Component for ScriptComponent {
    fn get_input_channels(&self) -> u64 {
        if let Value::Closure(ids, _env, box _body) = &self.val {
            ids.len() as u64
        } else {
            0
        }
    }

    fn get_output_channels(&self) -> u64 {
        //todo!
        2
    }

    fn prepare_play(&mut self, info: &PlaybackInfo) {
        //do nothing
    }

    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        let mut info = info.clone(); //todo: inefficient
        for (_count, out_per_channel) in output
            .chunks_mut(self.get_output_channels() as usize)
            .enumerate()
        {
            info.current_time += 1;
            for (ch, s) in out_per_channel.iter_mut().enumerate() {
                if ch == 0 {
                    *s = self.compute_sample(&info) as f32;
                } else {
                    *s = 0.0
                }
            }
        }
    }
}

pub fn get_component_for_value(v: &script::Value) -> Box<dyn Component + Send + Sync> {
    let generator = ScriptComponent::try_new(v).expect("not a valid component");
    Box::new(generator)
    // match v {
    //     Value::Closure(
    //         _ids,
    //         _env,
    //         box Expr::App(box Expr::Literal(Value::ExtFunction(fname)), args),
    //     ) => match (fname.as_str(), &args.as_slice()) {
    //         (
    //             "sinewave",
    //             &[Expr::Literal(Value::Parameter(freq)), Expr::Literal(Value::Parameter(amp)), Expr::Literal(Value::Parameter(phase))],
    //         ) => Box::new(oscillator::sinewave(OscillatorParam {
    //             amp: amp.clone(),
    //             freq: freq.clone(),
    //             phase: phase.clone(),
    //         })),
    //         (
    //             "sawtooth",
    //             &[Expr::Literal(Value::Parameter(freq)), Expr::Literal(Value::Parameter(amp)), Expr::Literal(Value::Parameter(phase)), Expr::Literal(Value::Parameter(dir))],
    //         ) => Box::new(oscillator::saw(
    //             OscillatorParam {
    //                 amp: amp.clone(),
    //                 freq: freq.clone(),
    //                 phase: phase.clone(),
    //             },
    //             dir.clone(),
    //         )),
    //         (
    //             "rectangular",
    //             &[Expr::Literal(Value::Parameter(freq)), Expr::Literal(Value::Parameter(amp)), Expr::Literal(Value::Parameter(phase)), Expr::Literal(Value::Parameter(duty))],
    //         ) => Box::new(oscillator::rect(
    //             OscillatorParam {
    //                 amp: amp.clone(),
    //                 freq: freq.clone(),
    //                 phase: phase.clone(),
    //             },
    //             duty.clone(),
    //         )),
    //         (
    //             "triangular",
    //             &[Expr::Literal(Value::Parameter(freq)), Expr::Literal(Value::Parameter(amp)), Expr::Literal(Value::Parameter(phase))],
    //         ) => Box::new(oscillator::sinewave(OscillatorParam {
    //             amp: amp.clone(),
    //             freq: freq.clone(),
    //             phase: phase.clone(),
    //         })),
    //         ("constant", &[Expr::Literal(Value::Parameter(val))]) => {
    //             Box::new(constant::Constant(val.clone()))
    //         }
    //         #[cfg(not(target_arch = "wasm32"))]
    //         ("fileplayer", &[Expr::Literal(Value::String(path))]) => {
    //             let p = FilePlayerParam {
    //                 path: path.clone(),
    //                 channels: UIntParameter::new(2, "channels").set_range(0..=2),
    //                 start_sec: FloatParameter::new(0.0, "start").set_range(0.0..=10.0),
    //                 duration: FloatParameter::new(1.0, "duration").set_range(0.0..=10.0),
    //             };
    //             Box::new(fileplayer::FilePlayer::new(Arc::new(p)))
    //         }
    //         (_, _) => {
    //             panic!("No matching generator")
    //         }
    //     },
    //     _ => panic!("invalid components"),
    // }
}


/// Interface for offline rendering.
pub trait RangedComponent: std::fmt::Debug {
    fn get_range(&self) -> RangeInclusive<f64>;
    fn get_output_channels(&self) -> u64;
    fn render_offline(&mut self, dest: &mut [f32], sample_rate: u32, channels: u64);
}


#[derive(Debug)]
pub struct RangedComponentDyn {
    generator: Box<dyn Component + Sync + Send>,
    range: AtomicRange<f64>,
    // buffer: Vec<f32>,
}

impl RangedComponentDyn {
    pub fn new(generator: Box<dyn Component + Sync + Send>, range: AtomicRange<f64>) -> Self {
        Self {
            generator,
            range,
            // buffer: vec![],
        }
    }
}

impl RangedComponent for RangedComponentDyn {
    fn get_range(&self) -> RangeInclusive<f64> {
        let (start, end) = self.range.get_pair();
        start..=end
    }

    fn get_output_channels(&self) -> u64 {
        self.generator.get_output_channels()
    }

    fn render_offline(&mut self, dest: &mut [f32], sample_rate: u32, channels: u64) {
        let info_local = PlaybackInfo {
            sample_rate,
            current_time: 0,
            frame_per_buffer: dest.len() as u64 / channels,
            channels,
        };
        // self.buffer.resize(
        //     (self.range.getrange() * sample_rate as f64) as usize * channels as usize,
        //     0.0,
        // );
        let input_dummy = vec![0.0f32; 1];
        self.generator.prepare_play(&info_local);
        self.generator.render(&input_dummy, dest, &info_local)
    }
}
