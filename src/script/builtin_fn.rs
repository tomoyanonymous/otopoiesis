use crate::{
    audio::PlaybackInfo,
    data::{AppModel, Content, FadeParam, Region, RegionFilter},
};

use super::{Environment, EvalError, Expr, ExtFun, ExtFunT, Type, Value};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ArrayReverse {}

impl ExtFunT for ArrayReverse {
    fn exec(
        &self,
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
        app: &mut Option<&mut AppModel>,
        play_info: &Option<&PlaybackInfo>,
        v: &[Value],
    ) -> Result<Value, EvalError> {
        // ここでは実際のフェードイン、アウトはしない。
        // プロジェクト生成時にリージョン→リージョンの変換をする際、リージョンの長さが変化する場合はこの関数内で操作する、ということになる。
        if v.len() != 3 {
            return Err(EvalError::InvalidNumArgs(3, v.len()));
        }
        match v {
            [region, Value::Parameter(time_in), Value::Parameter(time_out)] => {
                let mut rg = Region::try_from(region).expect("not a region");

                let label = rg.label.clone();
                let content = Value::new_lazy(Expr::App(
                    Expr::Var("apply_fade_in_out".into()).into(),
                    vec![
                        Expr::Literal(region.clone()),
                        Expr::Literal(Value::Parameter(time_in.clone())),
                        Expr::Literal(Value::Parameter(time_out.clone())),
                    ],
                ));
                Ok(Value::Region(
                    time_in.clone(),
                    time_out.clone(),
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
pub struct ApplyFadeInOut {}
impl ExtFunT for ApplyFadeInOut {
    fn exec(
        &self,
        _app: &mut Option<&mut AppModel>,
        _play_info: &Option<&PlaybackInfo>,
        _v: &[Value],
    ) -> Result<Value, EvalError> {
        // do nothing for now
        Err(EvalError::NotFound)
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
        app: &mut Option<&mut crate::data::AppModel>,
        play_info: &Option<&PlaybackInfo>,
        v: &[Value],
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
