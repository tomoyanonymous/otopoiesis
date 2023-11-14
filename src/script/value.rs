use crate::{data::AppModel, parameter::Parameter};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    None,
    Number(f64),
    Parameter(Arc<FloatParameter>), //shared through
    String(String),
    Array(Vec<Value>, Type), //typed array
    Function(Vec<Id>, Box<Expr>),
    Closure(Vec<Id>, Arc<Environment<Value>>, Box<Expr>),
    ExtFunction(ExtFun),
    Track(Box<Value>, Type),                //input type, output type
    Region(Arc<FloatParameter>,Arc<FloatParameter>, Box<Value>, Id, Type), //start,dur,content,label,type
    Project(f64, Vec<Value>),               //todo:reducer
}

impl Value {
    pub fn new_lazy(expr: Expr) -> Self {
        //wrap expression with function without arguments
        Self::Closure(
            vec![],
            Arc::new(builtin_fn::gen_global_env()),
            Box::new(expr),
        )
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
            _ => Err(EvalError::TypeMismatch("not a float".into())),
        }
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
