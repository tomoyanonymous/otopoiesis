use std::ops::{Deref, DerefMut};

use super::Rate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TypeRef(pub Box<Type>);
impl From<Type> for TypeRef {
    fn from(value: Type) -> Self {
        Self(Box::new(value))
    }
}
impl AsRef<Type> for TypeRef {
    fn as_ref(&self) -> &Type {
        &self.0
    }
}
impl Deref for TypeRef {
    type Target = Type;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type {
    Unknown,
    Unit,
    Number,
    Int,
    String,
    Tuple(Vec<TypeRef>),
    Array(TypeRef, u64),             //type, number of element
    Function(Vec<TypeRef>, TypeRef), //from,to
    Event(TypeRef),                  //type
    Vec(TypeRef),                    //type,
    IVec(TypeRef, Rate),             //type, sample_rate
}
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Tuple(l0), Self::Tuple(r0)) => l0 == r0,
            (Self::Array(l0, l1), Self::Array(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Function(l0, l1), Self::Function(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Event(l0), Self::Event(r0)) => l0 == r0,
            (Self::Vec(l0), Self::Vec(r0)) => l0 == r0,
            (Self::IVec(l0, l1), Self::IVec(r0, r1)) => l0 == r0 && l1 == r1,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
impl Type {
    pub fn midi_note() -> Self {
        Self::Event(Self::Tuple(vec![Type::Int.into(), Type::Int.into(), Type::Int.into()]).into())
    }
}
