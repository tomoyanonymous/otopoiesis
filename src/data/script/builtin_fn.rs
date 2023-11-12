use super::{EvalError, ExtFun, ExtFunT, Value};
use std::collections::HashMap;


#[derive(Clone, Debug)]
pub struct ArrayReverse {}

impl ExtFunT for ArrayReverse {
    fn exec(&self, _app: & crate::data::AppModel, v: &Value) -> Result<Value, EvalError> {
        match v {
            Value::Array(a, t) => {
                let mut res = a.clone();
                res.reverse();
                Ok(Value::Array(res, t.clone()))
            }
            _ => Err(EvalError::TypeMismatch("Not an array".into())),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Generator {}

pub fn gen_default_functions() -> HashMap<&'static str, ExtFun> {
    HashMap::from([("reverse", ExtFun::new(ArrayReverse{}))])
}
