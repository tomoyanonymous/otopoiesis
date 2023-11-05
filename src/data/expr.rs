use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::types::VType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value {
    Real(f64),
    Int(i64),
    Vector(Box<Expr>, u64),
    Function(Vec<Id>, Box<Expr>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Infix {
    Add,
    Sub,
    Mult,
    Div,
    Modulo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Unary {
    Negate,
    Sin,
    Cos,
    Sqrt,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Id(String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Expr {
    //literal types
    Real(f64),
    Region(Box<Expr>, Box<Expr>, Box<Expr>), //start,dur,generator
    Track(VType, VType, Box<Expr>),          //input,output,
    Project(Box<Expr>, Box<Expr>),           //array of tracks, reducer(mixer)
    //other operations
    Param(Id), //exposed to ui
    Var(Id),
    Lambda(Vec<Id>, Box<Expr>),
    App(Box<Expr>, Vec<Expr>),
    InfixOp(Infix, Box<Expr>, Box<Expr>),
    UnaryOp(Unary, Box<Expr>),
}

#[derive(Clone, Debug)]
pub struct EvalEnv {
    pub local: HashMap<String, Value>,
    pub global: HashMap<String, Value>,
}
impl EvalEnv {
    pub fn new() -> Self {
        Self {
            local: HashMap::new(),
            global: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum EvalError {
    VariableNotFound(String),
    NonCallable,
    NotANumber,
}

impl Infix {
    pub fn eval(&self, lhs: &Expr, rhs: &Expr, env: &EvalEnv) -> Result<Value, EvalError> {
        let l = lhs.eval(env)?;
        let r = rhs.eval(env)?;
        if let (Value::Real(lv), Value::Real(rv)) = (l, r) {
            Ok(Value::Real(match self {
                Infix::Add => lv + rv,
                Infix::Sub => lv - rv,
                Infix::Mult => lv * rv,
                Infix::Div => lv / rv,
                Infix::Modulo => lv % rv,
            }))
        } else {
            Err(EvalError::NotANumber)
        }
    }
}

impl Unary {
    pub fn eval(&self, rhs: &Expr, env: &EvalEnv) -> Result<Value, EvalError> {
        let r = rhs.eval(env)?;
        if let Value::Real(rv) = r {
            Ok(Value::Real(match self {
                Unary::Negate => -rv,
                Unary::Sin => rv.sin(),
                Unary::Cos => rv.cos(),
                Unary::Sqrt => rv.sqrt(),
            }))
        } else {
            Err(EvalError::NotANumber)
        }
    }
}

impl Expr {
    pub fn eval(&self, env: &EvalEnv) -> Result<Value, EvalError> {
        match self {
            Expr::Real(v) => Ok(Value::Real(*v)),
            Expr::Var(n) => match env.local.get(&n.0) {
                Some(v) => Ok(v.clone()),
                None => Err(EvalError::VariableNotFound(n.0.clone())),
            },
            Expr::Param(_) => todo!(),
            Expr::Lambda(id, body) => Ok(Value::Function(id.clone(), body.clone())),
            Expr::App(callee, arg) => match callee.eval(env)? {
                Value::Function(id, body) => {
                    let a: Vec<_> = arg.iter().map(|a| a.eval(env)).collect();
                    let mut env = env.clone();
                    id.iter().zip(a.iter()).for_each(|(i, a)| {
                        if a.is_ok() {
                            env.local.insert(i.0.clone(), a.as_ref().unwrap().clone());
                        } else {
                            a.as_ref().unwrap_err().clone();
                        }
                    });
                    body.eval(&env)
                }
                _ => Err(EvalError::NonCallable),
            },
            Expr::InfixOp(op, lhs, rhs) => op.eval(lhs, rhs, env),
            Expr::UnaryOp(op, rhs) => op.eval(rhs, env),
            Expr::Region(_, _, _) => todo!(),
            Expr::Track(_, _, _) => todo!(),
            Expr::Project(_, _) => todo!(),
        }
    }
}

pub struct Project {
    pub sample_rate: f64,
    pub track: Expr,
    pub mixer: Expr,
}

impl Project {
    // pub fn eval(&mut self, env: EvalEnv) -> Result<Value, EvalError> {
    //     let tracks = self.track.eval(&env)?;

    //     self.mixer
    // }
}
