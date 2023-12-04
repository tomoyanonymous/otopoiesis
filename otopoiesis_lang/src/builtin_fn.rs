use id_arena::Id;

use crate::{
    compiler::Context,
    expr::ExprRef,
    parameter::{FloatParameter, Parameter, RangedNumeric},
    runtime::PlayInfo,
    value::{self, RawValue, Region},
};

use super::{Environment, EvalError, Expr, ExtFun, ExtFunT, Type, Value};
pub mod loadwav;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ArrayReverse {}

impl ExtFunT for ArrayReverse {
    fn exec(
        &self,
        _play_info: &Option<&Box<dyn PlayInfo + Send + Sync>>,
        ctx: &mut Context,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        todo!()
        // if v.len() != 1 {
        //     return Err(EvalError::InvalidNumArgs(1, v.len()));
        // }
        // match v.get(0).unwrap() {
        //     Value::Array(a, t) => {
        //         let mut res = a.clone();
        //         res.reverse();
        //         Ok(Value::Array(res, t.clone()))
        //     }
        //     _ => Err(EvalError::TypeMismatch("Not an array".into())),
        // }
    }

    fn get_name(&self) -> &str {
        "array_reverse"
    }

    fn get_params(&self) -> &[String] {
        &[]
    }
}

#[derive(Clone, Debug)]
pub struct Print {
    // params: Vec<Param>,
}

impl ExtFunT for Print {
    fn exec(
        &self,
        _play_info: &Option<&Box<dyn PlayInfo + Send + Sync>>,
        ctx: &mut Context,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        //todo! generics
        // let str = v
        //     .iter()
        //     .fold(String::new(), |acc, b| format!("{}, {:?}", acc, b));
        // println!("{}", str);
        Ok(RawValue(0))
    }

    fn get_name(&self) -> &str {
        "print"
    }

    fn get_params(&self) -> &[String] {
        &[]
    }
}

#[derive(Clone, Debug)]
pub struct SineWave {}

impl ExtFunT for SineWave {
    fn exec(
        &self,
        play_info: &Option<&Box<dyn PlayInfo + Send + Sync>>,
        ctx: &mut Context,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        match play_info {
            Some(info) => match &v {
                &[freq, amp, phase] => {
                    //2Hzなら (now/sr)
                    let now = info.get_current_time_in_sample();
                    let now_s = now as f64 / info.get_samplerate();
                    let f = freq.get_as_float();
                    let a = amp.get_as_float();
                    let p = phase.get_as_float();
                    let phase_sample = f * now_s + p;
                    Ok(RawValue::from(
                        (phase_sample * std::f64::consts::PI * 2.0).sin() * a,
                    ))
                }
                _ => Err(EvalError::InvalidNumArgs(3, v.len())),
            },
            None => Err(EvalError::NotInPlayMode),
        }
    }

    fn get_name(&self) -> &str {
        "sinewave"
    }

    fn get_params(&self) -> &[String] {
        &[]
    }
}

#[derive(Debug)]
pub struct FadeInOut {}

impl ExtFunT for FadeInOut {
    fn exec(
        &self,
        _play_info: &Option<&Box<dyn PlayInfo + Send + Sync>>,
        ctx: &mut Context,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        // ここでは実際のフェードイン、アウトはしない。
        // プロジェクト生成時にリージョン→リージョンの変換をする際、リージョンの長さが変化する場合はこの関数内で操作する、ということになる。
        if v.len() != 3 {
            return Err(EvalError::InvalidNumArgs(3, v.len()));
        }
        match v {
            [origin, time_in, time_out] => {
                let value::Region(envid, start, dur, content) =
                    origin.get_as_ref::<&value::Region>();
                let env = ctx.env_storage.extend(envid.clone(), &[]);
                let app = ExprRef(ctx.expr_storage.alloc(Expr::AppExt(
                    ExtFun(Arc::new(ApplyFadeInOut {
                        time_in: *time_in,
                        time_out: *time_out,
                    })),
                    vec![content.clone(), start.clone(), dur.clone()],
                )));
                // let content = ctx.gen_closure(env, &vec![], &ExprRef(app));
                let region = value::Region(env, start.clone(), dur.clone(), app);
                let region = ctx.region_storage.alloc(region);
                let region_ref = ctx.region_storage.get_mut(region).unwrap();
                Ok(RawValue::from(region_ref as *mut Region))
            }
            _ => Err(EvalError::TypeMismatch("argument type mismatch".into())),
        }
    }

    fn get_name(&self) -> &str {
        "fadeinout"
    }

    fn get_params(&self) -> &[String] {
        &[]
    }
}

#[derive(Clone, Debug)]
struct FadeInfo<'a> {
    start: &'a u64,
    dur: &'a u64,
    time_in: &'a u64,
    time_out: &'a u64,
}

#[derive(Clone, Debug)]
enum FadeState {
    BeforeRange,
    FadeIn(f64),
    NonFade,
    FadeOut(f64),
    AfterRange,
}
impl FadeState {
    pub fn get_gain(&self) -> f64 {
        match self {
            FadeState::BeforeRange | FadeState::AfterRange => 0.0,
            FadeState::NonFade => 1.0,
            FadeState::FadeIn(g) | FadeState::FadeOut(g) => *g,
        }
    }
}

impl<'a> FadeInfo<'a> {
    pub fn new(start: &'a u64, dur: &'a u64, time_in: &'a u64, time_out: &'a u64) -> Option<Self> {
        if time_in + time_out <= *dur {
            Some(Self {
                start,
                dur,
                time_in,
                time_out,
            })
        } else {
            None
        }
    }
    pub fn calc(&self, now: u64) -> FadeState {
        let reltime = now as i64 - *self.start as i64;
        if reltime <= 0 {
            return FadeState::BeforeRange;
        }
        if reltime >= *self.dur as i64 {
            return FadeState::AfterRange;
        }
        if reltime < *self.time_in as i64 {
            let gain = reltime as f64 / *self.time_in as f64;

            return FadeState::FadeIn(gain);
        }
        let out_start = *self.dur as i64 - *self.time_out as i64;
        if reltime > out_start {
            let gain = 1.0 - (reltime - out_start) as f64 / *self.time_out as f64;
            return FadeState::FadeOut(gain);
        } else {
            return FadeState::NonFade;
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApplyFadeInOut {
    pub time_in: RawValue,
    pub time_out: RawValue,
}
impl ApplyFadeInOut {}
impl ApplyFadeInOut {
    pub fn apply(
        input: f64,
        now: u64,
        start: u64,
        dur: u64,
        time_in: u64,
        time_out: u64,
    ) -> Result<f64, EvalError> {
        let fadeinfo = FadeInfo::new(&start, &dur, &time_in, &time_out);
        fadeinfo
            .map(|info| info.calc(now).get_gain() * input)
            .ok_or(EvalError::InvalidConversion)
    }
}
impl ExtFunT for ApplyFadeInOut {
    fn exec(
        &self,
        play_info: &Option<&Box<dyn PlayInfo + Send + Sync>>,
        ctx: &mut Context,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        let now = play_info.unwrap().get_current_time_in_sample();
        let sr = play_info.unwrap().get_samplerate();
        // do nothing for now
        match v {
            [input_sample, _start, dur, time_in, time_out] => {
                let input = input_sample.get_as_float();
                let start = 0;
                let dur = (dur.get_as_float() * sr) as u64;
                let time_in = (time_in.get_as_float() * sr) as u64;
                let time_out = (time_out.get_as_float() * sr) as u64;
                // Ok(Value::Number(input))//なんかおかしい
                Self::apply(input, now as u64, start, dur, time_in, time_out)
                    .map(|res| RawValue::from(res))
            }
            _ => Err(EvalError::InvalidNumArgs(5, v.len())),
        }
    }

    fn get_name(&self) -> &str {
        "apply_fade_in_out"
    }

    fn get_params(&self) -> &[String] {
        &[]
    }
}

#[derive(Debug)]
pub struct Nop {}

impl ExtFunT for Nop {
    fn exec(
        &self,
        _play_info: &Option<&Box<dyn PlayInfo + Send + Sync>>,
        ctx: &mut Context,
        _v: &[Value],
    ) -> Result<Value, EvalError> {
        Ok(RawValue(0))
    }

    fn get_name(&self) -> &str {
        "nop"
    }

    fn get_params(&self) -> &[String] {
        &[]
    }
}
// pub fn lookup_extfun(name: &str, env: &Arc<Environment>) -> Result<ExtFun, EvalError> {
//     match name {
//         "sinewave" => Ok(ExtFun::new(SineWave::new())),
//         "fadeinout" => Ok(ExtFun::new(FadeInOut::new(env))),
//         "apply_fade_in_out" => Ok(ExtFun::new(ApplyFadeInOut::new())),
//         // "fileplayer"=>Ok(ExtFun::new(Nop{})),
//         _ => Err(EvalError::NotFound),
//     }
// }
pub fn gen_default_functions() -> Vec<(String, ExtFun)> {
    vec![
        ("reverse".into(), ExtFun::new(ArrayReverse {})),
        ("sinewave".into(), ExtFun::new(SineWave {})),
        ("fadeinout".into(), ExtFun::new(FadeInOut {})),
    ]
}
// pub fn gen_global_env() -> Environment {
// let v = gen_default_functions()
//     .iter()
//     .map(|(s, f)| (s.clone(), Value::ExtFunction(f.clone())))
//     .collect::<Vec<_>>();
// let mut env = Environment::new();
// env.local = v;
// env
// }
