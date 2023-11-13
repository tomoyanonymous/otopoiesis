use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{data, parameter::FloatParameter};

pub mod builtin_fn;
mod test;
// use serde::{Deserialize, Serialize};
pub trait ExtFunT: std::fmt::Debug {
    fn exec(&self, app: &mut data::AppModel, v: &Vec<Value>) -> Result<Value, EvalError>;
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
    Parameter(Arc<FloatParameter>), //shared through
    String(String),
    Array(Vec<Value>, Type), //typed array
    Function(Vec<Id>, Box<Expr>),
    Closure(Vec<Id>, Arc<Environment<Value>>, Box<Expr>),
    ExtFunction(Id),
    Track(Box<Value>, Type),                //input type, output type
    Region(f64, f64, Box<Value>, Id, Type), //start,dur,content,label,type
    Project(f64, Vec<Value>),               //todo:reducer
}

impl Value {
    pub fn new_lazy(expr:Expr)->Self{
        Self::Function(vec![],Box::new(expr))
    }
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
            Value::Number(_) | Value::Parameter(_) => Type::Number,
            Value::String(_) => Type::String,
            Value::Array(v, t) => {
                // let _t_elem = v.get(0).map_or(Type::Unknown, |v| v.get_type()).into();
                // assert_eq!(t, _t_elem);
                Type::Array(Box::new(t.clone()), v.len() as u64)
            }
            Value::Function(_a, _v) => todo!(),
            Value::Closure(_, _, _) => todo!(),
            Value::ExtFunction(_f) => Type::Function(Type::Unknown.into(), Type::Unknown.into()), //cannot infer?
            Value::Track(_input, _output) => todo!(),
            Value::Region(_start, _dur, _, _label, _) => todo!(),
            Value::Project(_sr, _tracks) => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Var(Id),
    Let(Id, Box<Expr>, Box<Expr>),
    Lambda(Vec<Id>, Box<Expr>),
    App(Box<Expr>, Vec<Expr>), //currently only single argument
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Environment<T>
where
    T: Clone,
{
    pub local: Vec<(Id, T)>,
    pub parent: Option<Arc<Self>>,
}

impl<T> Environment<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self {
            local: vec![],
            parent: None,
        }
    }
    pub fn bind(&mut self, key: &Id, val: T) {
        self.local.push((key.clone(), val.clone()))
    }
    pub fn lookup(&self, key: &Id) -> Option<&T> {
        self.local
            .iter()
            .find_map(|e| if &e.0 == key { Some(&e.1) } else { None })
            .or_else(|| self.parent.as_ref().map(|e| e.lookup(key)).flatten())
    }
}
pub fn extend_env<T: Clone>(env: Arc<Environment<T>>) -> Environment<T> {
    Environment::<T> {
        local: vec![],
        parent: Some(Arc::clone(&env)),
    }
}

pub enum EvalError {
    TypeMismatch(String),
    NotFound,
    InvalidNumArgs(usize, usize), //expected,actual
}

impl Expr {
    pub fn eval(
        &self,
        env: Arc<Environment<Value>>,
        app: &mut data::AppModel,
    ) -> Result<Value, EvalError> {
        match self {
            Expr::Literal(v) => Ok(v.clone()),
            Expr::Var(v) => env.lookup(v).ok_or(EvalError::NotFound).cloned(),
            Expr::Lambda(ids, body) => Ok(Value::Closure(ids.clone(), env.clone(), body.clone())),
            Expr::Let(id, body, then) => {
                let mut newenv = extend_env(env.clone());
                let body_v = body.eval(env, app)?;
                newenv.bind(id, body_v);
                then.eval(Arc::new(newenv), app)
            }
            Expr::App(fe, args) => {
                let f = fe.eval(env.clone(), app)?;
                let mut arg_res = vec![];
                for a in args.iter() {
                    match a.eval(env.clone(), app) {
                        Ok(res) => {
                            arg_res.push(res);
                        }
                        Err(e) => return Err(e),
                    }
                }
                match f {
                    Value::Function(_ids, _body) => {
                        todo!()
                    }
                    Value::Closure(ids, env, body) => {
                        let mut newenv = extend_env(env);
                        ids.iter().zip(arg_res.iter()).for_each(|(id, a)| {
                            newenv.bind(id, a.clone());
                        });
                        body.eval(Arc::new(newenv), app)
                    }
                    Value::ExtFunction(fname) => {
                        let f = app
                            .get_builtin_fn(&fname)
                            .ok_or(EvalError::NotFound)?
                            .clone();
                        f.0.exec(app, &arg_res)
                    }
                    _ => Err(EvalError::TypeMismatch("Not a Function".into())),
                }
            }
        }
    }
}
