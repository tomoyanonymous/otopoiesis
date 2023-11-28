use crate::{
    audio::PlaybackInfo,
    data::AppModel,
    parameter::{FloatParameter, Parameter, RangedNumeric},
};

use super::{extend_env, value::Param, Environment, EvalError, Expr, ExtFun, ExtFunT, Type, Value};
pub mod loadwav;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ArrayReverse {}

impl ExtFunT for ArrayReverse {
    fn exec(
        &self,
        _env: &Arc<Environment>,
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

    fn get_params(&self) -> &[super::value::Param] {
        &[]
    }
}

#[derive(Clone, Debug)]
pub struct Print {
    params: Vec<Param>,
}

impl ExtFunT for Print {
    fn exec(
        &self,
        _env: &Arc<Environment>,
        _app: &mut Option<&mut AppModel>,
        _play_info: &Option<&PlaybackInfo>,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        let str = v
            .iter()
            .fold(String::new(), |acc, b| format!("{}, {:?}", acc, b));
        println!("{}", str);
        Ok(Value::None)
    }

    fn get_name(&self) -> &str {
        "print"
    }

    fn get_params(&self) -> &[Param] {
        &self.params
    }
}

#[derive(Clone, Debug)]
pub struct SineWave {
    params: [Param; 3],
}
impl SineWave {
    pub fn new() -> Self {
        let freq = Param::Number(Arc::new(
            FloatParameter::new(440., "frequency").set_range(20.0..=20000.),
        ));
        let amp = Param::Number(Arc::new(
            FloatParameter::new(1.0, "amplitude").set_range(0.0..=1.0),
        ));
        let phase = Param::Number(Arc::new(
            FloatParameter::new(0.0, "phase").set_range(0.0..=1.0),
        ));
        Self {
            params: [freq, amp, phase],
        }
    }
}
impl ExtFunT for SineWave {
    fn exec(
        &self,
        _env: &Arc<Environment>,
        _app: &mut Option<&mut AppModel>,
        play_info: &Option<&PlaybackInfo>,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        match play_info {
            Some(info) => match &v {
                &[freq, amp, phase] => {
                    let res = {
                        //2Hzなら (now/sr)
                        let now = info.current_time;
                        let now_s = now as f64 / info.sample_rate as f64;
                        let f = freq.get_as_float()?;
                        let a = amp.get_as_float()?;
                        let p = phase.get_as_float()?;
                        let phase_sample = f * now_s + p;
                        Some((phase_sample * std::f64::consts::PI * 2.0).sin() * a)
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

    fn get_params(&self) -> &[Param] {
        &self.params
    }
}

#[derive(Debug)]
pub struct FadeInOut {
    params: [Param; 2],
}
impl FadeInOut {
    pub fn new() -> Self {
        let time_in = Param::Number(Arc::new(
            FloatParameter::new(0.01, "fade_in").set_range(0.0..=10.0),
        ));
        let time_out = Param::Number(Arc::new(
            FloatParameter::new(0.01, "fade_out").set_range(0.0..=10.0),
        ));
        Self {
            params: [time_in, time_out],
        }
    }
}

impl ExtFunT for FadeInOut {
    fn exec(
        &self,
        env: &Arc<Environment>,
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
            [origin, time_in, time_out] => {
                let (start, dur, content, _label) = match origin {
                    Value::Region(_env, start, dur, content, label, _type) => {
                        (start, dur, content, label)
                    }
                    _ => panic!("not a region"),
                };
                let env = Arc::new(extend_env(env.clone()));
                let content = Box::new(Expr::Lambda(
                    vec![],
                    Expr::App(
                        Expr::Var("apply_fade_in_out".into()).into(),
                        vec![
                            *content.clone(),
                            *start.clone(),
                            *dur.clone(),
                            Expr::Literal(time_in.clone()),
                            Expr::Literal(time_out.clone()),
                        ],
                    )
                    .into(),
                ));
                Ok(Value::Region(
                    env.clone(),
                    start.clone(),
                    dur.clone(),
                    content,
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

    fn get_params(&self) -> &[Param] {
        &self.params
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
    params: [Param; 4],
}
impl ApplyFadeInOut {
    pub fn new() -> Self {
        let start = Param::Number(Arc::new(FloatParameter::new(0.01, "start")));
        let dur = Param::Number(Arc::new(FloatParameter::new(0.01, "dur")));
        let time_in = Param::Number(Arc::new(
            FloatParameter::new(0.01, "fade_in").set_range(0.0..=10.0),
        ));
        let time_out = Param::Number(Arc::new(
            FloatParameter::new(0.01, "fade_out").set_range(0.0..=10.0),
        ));
        Self {
            params: [start, dur, time_in, time_out],
        }
    }
}
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
        _env: &Arc<Environment>,
        app: &mut Option<&mut AppModel>,
        play_info: &Option<&PlaybackInfo>,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        let now = play_info.unwrap().current_time;
        let sr = play_info.unwrap().sample_rate as f64;
        // do nothing for now
        match v {
            [input_sample, _start, dur, time_in, time_out] => {
                let input = match input_sample {
                    Value::Number(n) => Ok(*n as f64),
                    Value::Parameter(p) => Ok(p.get() as f64),
                    Value::Closure(_ids, env, body) => {
                        let n = body.eval(env.clone(), play_info, app)?;
                        Ok(n.get_as_float()?)
                    }
                    _ => Err(EvalError::InvalidConversion),
                }?;
                // let input = input_sample
                //     .eval_closure(play_info, app)
                //     .map(|s| s.get_as_float().expect("not a float"))
                //     .expect("not a closure");
                let start = 0;
                let dur = (dur.get_as_float().unwrap() * sr) as u64;
                let time_in = (time_in.get_as_float().unwrap() * sr) as u64;
                let time_out = (time_out.get_as_float().unwrap() * sr) as u64;
                // Ok(Value::Number(input))//なんかおかしい
                Self::apply(input, now as u64, start, dur, time_in, time_out)
                    .map(|res| Value::Number(res))
            }
            _ => Err(EvalError::InvalidNumArgs(5, v.len())),
        }
    }

    fn get_name(&self) -> &str {
        "apply_fade_in_out"
    }

    fn get_params(&self) -> &[Param] {
        &self.params
    }
}

#[derive(Debug)]
pub struct Nop {}

impl ExtFunT for Nop {
    fn exec(
        &self,
        _env: &Arc<Environment>,
        _app: &mut Option<&mut crate::data::AppModel>,
        _play_info: &Option<&PlaybackInfo>,
        _v: &[Value],
    ) -> Result<Value, EvalError> {
        Ok(Value::None)
    }

    fn get_name(&self) -> &str {
        "nop"
    }

    fn get_params(&self) -> &[Param] {
        &[]
    }
}
pub fn lookup_extfun(name: &str) -> Result<ExtFun, EvalError> {
    match name {
        "sinewave" => Ok(ExtFun::new(SineWave::new())),
        "fadeinout" => Ok(ExtFun::new(FadeInOut::new())),
        "apply_fade_in_out" => Ok(ExtFun::new(ApplyFadeInOut::new())),
        // "fileplayer"=>Ok(ExtFun::new(Nop{})),
        _ => Err(EvalError::NotFound),
    }
}
// pub fn gen_default_functions() -> Vec<(String, ExtFun)> {
//     vec![
//         ("reverse".into(), ExtFun::new(ArrayReverse {})),
//         ("sinewave".into(), ExtFun::new(SineWave {})),
//         ("fadeinout".into(), ExtFun::new(FadeInOut {})),
//         ("apply_fade_in_out".into(), ExtFun::new(ApplyFadeInOut {})),
//     ]
// }
// pub fn gen_global_env() -> Environment {
// let v = gen_default_functions()
//     .iter()
//     .map(|(s, f)| (s.clone(), Value::ExtFunction(f.clone())))
//     .collect::<Vec<_>>();
// let mut env = Environment::new();
// env.local = v;
// env
// }
