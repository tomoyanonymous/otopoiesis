use std::sync::Mutex;

use crate::parameter::Parameter;

use super::expr::ExprRef;
use super::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct RawValue(pub u64);

impl RawValue {
    pub fn get_as_ref<T>(&self) -> &T {
        let ptr = self.0 as *mut T;
        unsafe { &*ptr }
    }
    pub fn get_as_float(&self) -> f64 {
        unsafe { std::mem::transmute::<Self, f64>(*self) }
    }
}

impl From<f64> for RawValue {
    fn from(value: f64) -> Self {
        unsafe { std::mem::transmute::<f64, Self>(value) }
    }
}
impl From<*const Arc<FloatParameter>> for RawValue {
    fn from(value: *const Arc<FloatParameter>) -> Self {
        unsafe { std::mem::transmute::<*const Arc<FloatParameter>, Self>(value) }
    }
}
impl From<*const Arc<Mutex<String>>> for RawValue {
    fn from(value: *const Arc<Mutex<String>>) -> Self {
        unsafe { std::mem::transmute::<*const Arc<Mutex<String>>, Self>(value) }
    }
}
impl From<*mut Closure> for RawValue {
    fn from(value: *mut Closure) -> Self {
        RawValue(value as u64)
    }
}
impl From<*mut ExtFun> for RawValue {
    fn from(value: *mut ExtFun) -> Self {
        RawValue(value as u64)
    }
}
impl From<*mut Vec<RawValue>> for RawValue {
    fn from(value: *mut Vec<RawValue>) -> Self {
        RawValue(value as u64)
    }
}
impl From<*mut Track> for RawValue {
    fn from(value: *mut Track) -> Self {
        RawValue(value as u64)
    }
}
impl From<*mut Region> for RawValue {
    fn from(value: *mut Region) -> Self {
        RawValue(value as u64)
    }
}
impl From<*mut Project> for RawValue {
    fn from(value: *mut Project) -> Self {
        RawValue(value as u64)
    }
}
pub struct Closure {
    pub env: Id<Environment>,
    pub ids: Vec<Symbol>,
    pub body: ExprRef,
}
impl Closure {
    pub fn new(env: Id<Environment>, ids: &Vec<Symbol>, body: ExprRef) -> Self {
        Self {
            env,
            ids: ids.clone(),
            body,
        }
    }
}

pub struct Track(pub Id<Environment>, pub ExprRef); //input type, output type
pub struct Region(pub Id<Environment>, pub ExprRef, pub ExprRef, pub ExprRef); //start,dur,content,label,type
pub struct Project(pub Id<Environment>, pub f64, pub Vec<ExprRef>);

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub enum Param {
//     Number(Arc<FloatParameter>),
//     String(Arc<Mutex<String>>),
// }
// impl Param {
//     pub fn get_label(&self) -> String {
//         match self {
//             Param::Number(p) => p.get_label().to_string(),
//             Param::String(p) => {
//                 if let Ok(ref s) = p.try_lock() {
//                     s.to_string()
//                 } else {
//                     "failed to lock thread".to_string()
//                 }
//             }
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub enum Value {
//     None,
//     Number(f64),
//     Parameter(Arc<FloatParameter>), //shared through
//     String(Arc<Mutex<String>>),
//     Array(Vec<Value>, Type), //typed array
//     Function(Vec<Param>, Box<Expr>),
//     Closure(Vec<Param>, Arc<Environment>, Box<Expr>),
//     ExtFunction(ExtFun),
//     Track(Arc<Environment>, Box<Expr>, Type), //input type, output type
//     Region(Arc<Environment>, Box<Expr>, Box<Expr>, Box<Expr>, Id, Type), //start,dur,content,label,type
//     Project(Arc<Environment>, f64, Vec<Expr>),                           //todo:reducer
// }

// impl Value {
//     pub fn new_param(p: FloatParameter) -> Self {
//         Self::Parameter(Arc::new(p))
//     }
//     pub fn new_lazy(expr: Expr) -> Self {
//         //wrap expression with function without arguments
//         Self::Closure(vec![], Arc::new(Environment::new()), Box::new(expr))
//     }
//     pub fn eval_closure(
//         &self,
//         play_info: &Option<&Box<dyn PlayInfo + Send + Sync>>,
//     ) -> Result<Self, EvalError> {
//         match self {
//             Self::Closure(_ids, env, expr) => expr.eval(env.clone(), play_info),

//             _ => Err(EvalError::TypeMismatch("Not a Closure".into())),
//         }
//     }
//     pub fn get_as_float(&self) -> Result<f64, EvalError> {
//         match self {
//             Value::Parameter(p) => Ok(p.get() as f64),
//             Value::Number(f) => Ok(*f),
//             Value::Closure(_ids, env, body) => {
//                 let res = body.eval(env.clone(), &None)?;
//                 res.get_as_float()
//             }
//             _ => Err(EvalError::TypeMismatch("not a float".into())),
//         }
//     }
//     pub fn get_type(&self) -> Type {
//         match self {
//             Value::None => Type::Unit,
//             Value::Number(_) | Value::Parameter(_) => Type::Number,
//             Value::String(_) => Type::String,
//             Value::Array(v, t) => {
//                 // let _t_elem = v.get(0).map_or(Type::Unknown, |v| v.get_type()).into();
//                 // assert_eq!(t, _t_elem);
//                 Type::Array(Box::new(t.clone()), v.len() as u64)
//             }
//             Value::Function(_a, _v) => todo!(),
//             Value::Closure(_, _, _) => todo!(),
//             Value::ExtFunction(_f) => Type::Function(Type::Unknown.into(), Type::Unknown.into()), //cannot infer?
//             Value::Track(_env, _input, _output) => todo!(),
//             Value::Region(_env, _start, _dur, _content, _label, _) => todo!(),
//             Value::Project(_env, _sr, _tracks) => todo!(),
//         }
//     }
// }
