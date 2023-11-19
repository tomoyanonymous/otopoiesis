use std::sync::Mutex;

use crate::{data::AppModel, parameter::Parameter};

use super::*;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Param {
    Number(Arc<FloatParameter>),
    String(Arc<Mutex<String>>),
}
impl Param {
    pub fn get_label(&self) -> String {
        match self {
            Param::Number(p) => p.get_label().to_string(),
            Param::String(p) => {
                if let Ok(ref s) = p.try_lock() {
                    s.to_string()
                } else {
                    "failed to lock thread".to_string()
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    None,
    Number(f64),
    Parameter(Arc<FloatParameter>), //shared through
    String(Arc<Mutex<String>>),
    Array(Vec<Value>, Type), //typed array
    Function(Vec<Param>, Box<Expr>),
    Closure(Vec<Param>, Arc<Environment>, Box<Expr>),
    ExtFunction(ExtFun),
    Track(Arc<Environment>, Box<Expr>, Type), //input type, output type
    Region(Arc<Environment>, Box<Expr>, Box<Expr>, Box<Expr>, Id, Type), //start,dur,content,label,type
    Project(Arc<Environment>, f64, Vec<Expr>),                           //todo:reducer
}

impl Value {
    pub fn new_lazy(expr: Expr) -> Self {
        //wrap expression with function without arguments
        Self::Closure(vec![], Arc::new(Environment::new()), Box::new(expr))
    }
    pub fn eval_closure(
        &self,
        play_info: &Option<&PlaybackInfo>,
        app: &mut Option<&mut AppModel>,
    ) -> Result<Self, EvalError> {
        match self {
            Self::Closure(_ids, env, expr) => expr.eval(env.clone(), play_info, app),

            _ => Err(EvalError::TypeMismatch("Not a Closure".into())),
        }
    }
    pub fn get_as_float(&self) -> Result<f64, EvalError> {
        match self {
            Value::Parameter(p) => Ok(p.get() as f64),
            Value::Number(f) => Ok(*f),
            Value::Closure(_ids, env, body) => {
                let res = body.eval(env.clone(), &None, &mut None)?;
                res.get_as_float()
            }
            _ => Err(EvalError::TypeMismatch("not a float".into())),
        }
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
            Value::Track(_env, _input, _output) => todo!(),
            Value::Region(_env, _start, _dur, _content, _label, _) => todo!(),
            Value::Project(_env, _sr, _tracks) => todo!(),
        }
    }
}
