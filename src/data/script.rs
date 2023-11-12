use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::data;

pub mod builtin_fn;
// use serde::{Deserialize, Serialize};
pub trait ExtFunT: std::fmt::Debug {
    fn exec(&self, app: & data::AppModel, v: &Value) -> Result<Value, EvalError>;
}

pub trait MixerT: std::fmt::Debug {
    fn exec(&self, app: &mut data::AppModel, tracks: &Vec<Value>) -> Result<Value, EvalError>;
}

#[derive(Debug, Clone)]
pub struct ExtFun(Arc<dyn ExtFunT>);

impl ExtFun {
    pub fn new(e: impl ExtFunT + 'static) -> Self {
        Self(Arc::new(e))
    }
}

pub type Mixer = Arc<dyn MixerT>;
pub type Id = String;
pub type Time = f64;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum Rate {
    Audio,            //
    UpSampled(u64),   //multipled by
    DownSampled(u64), //divided by
    Control(f64),     //event per seconds(Hz)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type {
    Unknown,
    Unit,
    Number,
    Int,
    String,
    Tuple(Vec<Type>),
    Array(Box<Type>, u64),          //type, number of element
    Function(Box<Type>, Box<Type>), //from,to
    Event(Box<Type>),               //type
    Vec(Box<Type>),                 //type,
    IVec(Box<Type>, Rate),          //type, sample_rate
}
impl Type {
    pub fn midi_note() -> Self {
        Self::Event(Self::Tuple(vec![Type::Int, Type::Int, Type::Int]).into())
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    None,
    Number(f64),
    String(String),
    Array(Vec<Value>, Type), //typed array
    Function(Vec<Id>, Box<Expr>),
    ExtFunction(Id),
    Track(Box<Value>, Type), //input type, output type
    Region(f64,f64,Box<Value>,Id,Type),//start,dur,content,label,type
    Project(f64,Vec<Value>), //todo:reducer
}

impl Value {
    pub fn audio_track(channels: u64) -> Self {
        let t = Type::IVec(
            Type::Array(Type::Number.into(), channels).into(),
            Rate::Audio,
        );
        let generator = Value::None;
        Self::Track(generator.into(), t)
    }
    pub fn midi_track() -> Self {
        Self::Track(Value::None.into(), Type::Vec(Type::midi_note().into()))
    }
    pub fn get_type(&self) -> Type {
        match self {
            Value::None => Type::Unit,
            Value::Number(_) => Type::Number,
            Value::String(_) => Type::String,
            Value::Array(v, t) => {
                // let _t_elem = v.get(0).map_or(Type::Unknown, |v| v.get_type()).into();
                // assert_eq!(t, _t_elem);
                Type::Array(Box::new(t.clone()), v.len() as u64)
            }
            Value::Function(a, v) => todo!(),
            Value::ExtFunction(f) => Type::Function(Type::Unknown.into(), Type::Unknown.into()), //cannot infer?
            Value::Track(input, output) => todo!(),
            Value::Region(_start,_dur,_,_label,_) => todo!(),
            Value::Project(_sr,_tracks) => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Var(Id),
    App(Box<Expr>, Box<Expr>), //currently only single argument
}

pub struct Environment<T>(pub Vec<(Id, T)>);

impl<T> Environment<T> {
    // pub fn lookup()
}
pub enum EvalError {
    TypeMismatch(String),
    NotFound,
}

impl Expr {
    pub fn eval(
        &self,
        env: &Environment<Value>,
        app: & data::AppModel,
    ) -> Result<Value, EvalError> {
        match self {
            Expr::Literal(v) => Ok(v.clone()),
            Expr::Var(_) => todo!(),
            Expr::App(fe, arg) => {
                let f = fe.eval(env, app)?;
                let a = arg.eval(env, app)?;
                match f {
                    Value::Function(_ids, _body) => {
                        todo!()
                    }
                    Value::ExtFunction(fname) => {
                        let f = app
                            .get_builtin_fn(&fname)
                            .ok_or(EvalError::NotFound)?
                            .clone();
                        f.0.exec(app, &a)
                    }
                    _ => Err(EvalError::TypeMismatch("Not a Function".into())),
                }
            }
        }
    }
}
