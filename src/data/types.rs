use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VType {
    Real,     // type for audio and time
    Duration, //internally same as Real but > 0
    Int,
    Tuple(Vec<VType>),
    Function(Vec<VType>, Box<VType>),
    Vector(Box<VType>, u64),
}

//basic temporal type constructor.
pub fn event_ty(ty: VType) -> VType {
    VType::Tuple(vec![VType::Real, ty])
}
pub fn vec_ty(ty: VType, n: u64) -> VType {
    VType::Vector(Box::new(ty), n)
}
pub fn ivec_ty(ty: VType) -> VType {
    //本当はステートフルなストリームだが、mimiumであれば問題なし
    VType::Function(vec![], Box::new(ty))
}

pub fn generator_ty(stty: VType) -> VType {
    ivec_ty(stty)
}
pub fn region_ty(stty: VType) -> VType {
    //start,duration,stream
    VType::Tuple(vec![VType::Real, VType::Duration, stty])
}

pub enum Type {
    Generator(VType),
    Track(VType, VType), //input type, output type
    Region(Box<Type>),
    Project(VType, VType), //contain multiple tracks
    Array(Box<Type>),
}

pub enum Value {
    Project(),
}
