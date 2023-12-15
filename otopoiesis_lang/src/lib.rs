#![feature(box_patterns)]
#![feature(iterator_try_collect)]
pub mod atomic;
pub mod error;
pub mod builtin_fn;
pub mod metadata;
pub mod expr;
pub mod environment;
pub mod types;
pub mod parameter;
pub mod value;

pub mod parser;
pub mod compiler;
pub mod typing;
pub mod runtime;

use compiler::Context;
use runtime::PlayInfo;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, sync::Weak};

use crate::parameter::FloatParameter;
use id_arena::Id;
use string_interner::{backend::StringBackend, StringInterner};

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Symbol(usize); //Symbol Trait is implemented on usize

pub(crate) type Interner = StringInterner<StringBackend<usize>>;

pub use {compiler::EvalError, environment::Environment, expr::Expr};
pub type Value = value::RawValue;

// mod test;
// use serde::{Deserialize, Serialize};
pub trait ExtFunT: std::fmt::Debug {
    fn exec(
        &self,
        play_info: &Option<&Box<dyn PlayInfo + Send + Sync>>,
        ctx: &mut Context,
        v: &[Value],
    ) -> Result<Value, EvalError>;
    fn get_name(&self) -> &str;
    fn get_params(&self) -> &[String];
}

// pub trait MixerT: std::fmt::Debug {
//     fn exec(&self, app: &mut data::AppModel, tracks: &[Value]) -> Result<Value, EvalError>;
// }

#[derive(Debug, Clone)]
pub struct ExtFun(Arc<dyn ExtFunT>);
unsafe impl Send for ExtFun {}
unsafe impl Sync for ExtFun {}

impl ExtFun {
    pub fn new(e: impl ExtFunT + 'static) -> Self {
        Self(Arc::new(e))
    }
    pub fn get_name(&self) -> &str {
        self.0.get_name()
    }
    pub fn get_params(&self) -> &[String] {
        self.0.get_params()
    }
}

impl Serialize for ExtFun {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.get_name())
    }
}
struct ExtFunVisitor {}
impl<'d> serde::de::Visitor<'d> for ExtFunVisitor {
    type Value = ExtFun;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("External Fun")
    }
    fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ExtFun(Arc::new(builtin_fn::Nop {})))
    }
}

impl<'d> Deserialize<'d> for ExtFun {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        deserializer.deserialize_str(ExtFunVisitor {})
    }
}

// pub type Mixer = Arc<dyn MixerT>;
pub type Time = f64;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub enum Rate {
    Audio,            //
    UpSampled(u64),   //multipled by
    DownSampled(u64), //divided by
    Control(f64),     //event per seconds(Hz)
}

