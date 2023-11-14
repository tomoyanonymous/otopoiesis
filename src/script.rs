use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{data, parameter::FloatParameter};

pub mod builtin_fn;
pub mod environment;
pub mod expr;
pub mod value;
pub use {
    environment::{extend_env, Environment},
    expr::{EvalError, Expr},
    value::Value,
};
// mod test;
// use serde::{Deserialize, Serialize};
pub trait ExtFunT: std::fmt::Debug {
    fn exec(
        &self,
        app: &mut Option<&mut data::AppModel>,
        v: &Vec<Value>,
    ) -> Result<Value, EvalError>;
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
