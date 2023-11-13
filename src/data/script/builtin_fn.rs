use super::{EvalError, ExtFun, ExtFunT, Value};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ArrayReverse {}

impl ExtFunT for ArrayReverse {
    fn exec(&self, _app: &mut crate::data::AppModel, v: &Vec<Value>) -> Result<Value, EvalError> {
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
}

#[derive(Clone, Debug)]
pub struct Print {}

impl ExtFunT for Print {
    fn exec(&self, _app: &mut crate::data::AppModel, v: &Vec<Value>) -> Result<Value, EvalError> {
        let str = v
            .iter()
            .fold(String::new(), |acc, b| format!("{}, {:?}", acc, b));
        println!("({})", str);
        Ok(Value::None)
    }
}

#[derive(Clone, Debug)]
pub struct Generator {}



pub fn gen_default_functions() -> HashMap<&'static str, ExtFun> {
    HashMap::from([("reverse", ExtFun::new(ArrayReverse {}))])
}
