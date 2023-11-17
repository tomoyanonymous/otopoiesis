use crate::{
    audio::PlaybackInfo,
    data::{AppModel, Region},
};

use super::{extend_env, Environment, EvalError, Expr, ExtFun, ExtFunT, Id, Type, Value};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ArrayReverse {}

impl ExtFunT for ArrayReverse {
    fn exec(
        &self,
        env: &Arc<Environment<Value>>,
        _app: &mut Option<&mut AppModel>,
        _play_info: &Option<&PlaybackInfo>,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        if v.len() != 1 {
            return Err(EvalError::InvalidNumArgs(1, v.len()));
        }
        match v.get(0).unwrap() {
            Value::Array(a, t) => {
                let mut res = a.clone();
                res.reverse();
                Ok(Value::Array(res, t.clone()))
            }
            _ => Err(EvalError::TypeMismatch("Not an array".into())),
        }
    }

    fn get_name(&self) -> &str {
        "array_reverse"
    }
}

#[derive(Clone, Debug)]
pub struct Print {}

impl ExtFunT for Print {
    fn exec(
        &self,
        env: &Arc<Environment<Value>>,
        _app: &mut Option<&mut AppModel>,
        _play_info: &Option<&PlaybackInfo>,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        let str = v
            .iter()
            .fold(String::new(), |acc, b| format!("{}, {:?}", acc, b));
        println!("({})", str);
        Ok(Value::None)
    }

    fn get_name(&self) -> &str {
        "print"
    }
}

#[derive(Clone, Debug)]
pub struct SineWave {}
impl ExtFunT for SineWave {
    fn exec(
        &self,
        env: &Arc<Environment<Value>>,
        _app: &mut Option<&mut AppModel>,
        play_info: &Option<&PlaybackInfo>,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        match play_info {
            Some(info) => match &v {
                &[freq, amp, phase] => {
                    let res = {
                        let now = info.current_time;
                        let f = freq.get_as_float()?;
                        let a = amp.get_as_float()?;
                        let p = phase.get_as_float()?;
                        let phase_sample = f * (now as f64 / info.sample_rate as f64) % 1.0 + p;
                        Some(phase_sample.sin() * a)
                    };
                    res.map(|r| Value::Number(r))
                        .ok_or(EvalError::TypeMismatch("sinewave type error".into()))
                }
                _ => Err(EvalError::InvalidNumArgs(3, v.len())),
            },
            None => Err(EvalError::NotInPlayMode),
        }
    }

    fn get_name(&self) -> &str {
        "sinewave"
    }
}

#[derive(Debug)]
pub struct FadeInOut {}

impl ExtFunT for FadeInOut {
    fn exec(
        &self,
        env: &Arc<Environment<Value>>,
        _app: &mut Option<&mut AppModel>,
        _play_info: &Option<&PlaybackInfo>,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        // ここでは実際のフェードイン、アウトはしない。
        // プロジェクト生成時にリージョン→リージョンの変換をする際、リージョンの長さが変化する場合はこの関数内で操作する、ということになる。
        if v.len() != 3 {
            return Err(EvalError::InvalidNumArgs(3, v.len()));
        }
        match v {
            [origin, Value::Parameter(time_in), Value::Parameter(time_out)] => {
                let (start, dur, content, _label) = match origin {
                    Value::Region(start, dur, content, label, _type) => {
                        (start, dur, content, label)
                    }
                    _ => panic!("not a region"),
                };
                let mut env = extend_env(env.clone());
                let content = Value::Closure(
                    vec![],
                    Arc::new(env),
                    Expr::App(
                        Expr::Var("apply_fade_in_out".into()).into(),
                        vec![
                            Expr::Literal(*content.clone()),
                            Expr::Literal(Value::Parameter(start.clone())),
                            Expr::Literal(Value::Parameter(dur.clone())),
                            Expr::Literal(Value::Parameter(time_in.clone())),
                            Expr::Literal(Value::Parameter(time_out.clone())),
                        ],
                    )
                    .into(),
                );
                Ok(Value::Region(
                    start.clone(),
                    dur.clone(),
                    content.into(),
                    "fade_in_out".into(),
                    Type::Unknown,
                ))
            }
            _ => Err(EvalError::TypeMismatch("argument type mismatch".into())),
        }
    }

    fn get_name(&self) -> &str {
        "fadeinout"
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
            return FadeState::FadeIn(reltime as f64 / *self.time_in as f64);
        }
        let out_start = *self.dur as i64 - *self.time_out as i64;
        if reltime > out_start {
            let ratio = (out_start - reltime) as f64 / *self.time_out as f64;
            return FadeState::FadeOut(ratio);
        } else {
            return FadeState::NonFade;
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApplyFadeInOut {}
impl ApplyFadeInOut {
    pub fn apply(input: f64, now: u64, start: u64, dur: u64, time_in: u64, time_out: u64) -> f64 {
        let fadeinfo = FadeInfo::new(&start, &dur, &time_in, &time_out);
        let gain = fadeinfo.map_or(0.0, |info| info.calc(now).get_gain());
        input * gain
    }
}
impl ExtFunT for ApplyFadeInOut {
    fn exec(
        &self,
        env: &Arc<Environment<Value>>,
        _app: &mut Option<&mut AppModel>,
        play_info: &Option<&PlaybackInfo>,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        let now = play_info.unwrap().current_time;
        let sr = play_info.unwrap().sample_rate as f64;
        // do nothing for now
        match v {
            [input_sample, start, dur, time_in, time_out] => {
                let input = input_sample
                    .eval_closure(play_info, _app)
                    .map(|s| s.get_as_float().expect("not a float"))
                    .expect("not a closure");
                let start = (start.get_as_float().unwrap_or(0.0) * sr) as u64;
                let dur = (dur.get_as_float().unwrap_or(0.0) * sr) as u64;
                let time_in = (time_in.get_as_float().unwrap_or(0.0) * sr) as u64;
                let time_out = (time_out.get_as_float().unwrap_or(0.0) * sr) as u64;
                Ok(Value::Number(input))//なんかおかしい
                // Ok(Value::Number(Self::apply(
                //     input, now as u64, start, dur, time_in, time_out,
                // )))
            }
            _ => Err(EvalError::InvalidNumArgs(5, v.len())),
        }
    }

    fn get_name(&self) -> &str {
        "apply_fade_in_out"
    }
}

#[derive(Debug)]
pub struct Nop {}

impl ExtFunT for Nop {
    fn exec(
        &self,
        _env: &Arc<Environment<Value>>,
        _app: &mut Option<&mut crate::data::AppModel>,
        _play_info: &Option<&PlaybackInfo>,
        _v: &[Value],
    ) -> Result<Value, EvalError> {
        Ok(Value::None)
    }

    fn get_name(&self) -> &str {
        "nop"
    }
}

pub fn gen_default_functions() -> Vec<(String, ExtFun)> {
    vec![
        ("reverse".into(), ExtFun::new(ArrayReverse {})),
        ("sinewave".into(), ExtFun::new(SineWave {})),
        ("fadeinout".into(), ExtFun::new(FadeInOut {})),
        ("apply_fade_in_out".into(), ExtFun::new(ApplyFadeInOut {})),
    ]
}
pub fn gen_global_env() -> Environment<Value> {
    let v = gen_default_functions()
        .iter()
        .map(|(s, f)| (s.clone(), Value::ExtFunction(f.clone())))
        .collect::<Vec<_>>();
    let mut env = Environment::new();
    env.local = v;
    env
}
