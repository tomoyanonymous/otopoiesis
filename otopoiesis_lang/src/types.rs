use std::rc::Rc;

use serde::{Serialize,Deserialize};
use super::Rate;

pub type TypeId =i64;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Type {
    Unknown,
    Unit,
    Number,
    Int,
    String,
    Tuple(Vec<Type>),
    Array(Box<Type>, u64),          //type, number of element
    Function(Vec<Type>, Box<Type>), //from,to
    Event(Box<Type>),               //type
    Vec(Box<Type>),                 //type,
    IVec(Box<Type>, Rate),          //type, sample_rate
    //(experimental) code-type for multi-stage computation that will be evaluated on the next stage
    Code(Box<Type>),
    Intermediate(TypeId),
}
impl Default for Type{
    fn default() -> Self {
        Type::Unknown
    }
}
impl Type {
    pub fn midi_note() -> Self {
        Self::Event(Self::Tuple(vec![Type::Int, Type::Int, Type::Int]).into())
    }

pub fn apply_fn<F>(&self, closure: F) -> Self
where
    F: Fn(Self) -> Self,
{
    let apply_box = |a: &Self| -> Box<Self> { Box::new(closure(a.clone())) };
    let apply_vec =
        |v: &Vec<Self>| -> Vec<Self> { v.iter().map(|a| closure(a.clone())).collect() };
    match self {
        Type::Array(a,l) => Type::Array(apply_box(a),*l),
        Type::Tuple(v) => Type::Tuple(apply_vec(v)),
        Type::Function(p, r,) => {
            Type::Function(apply_vec(p), apply_box(r))
        }
        Type::Code(c) => Type::Code(apply_box(c)),
        Type::Intermediate(id) => Type::Intermediate(*id),
        _ => self.clone(),
    }
}
}
