use atomic_float::AtomicF64;
use std::sync::{atomic::AtomicU64, Arc};
pub trait DataWrapper {
    type Primitive;
    type Shared;
}

pub trait Collection<T> {
    type Item;
}

pub struct UInt<T>;

impl<T> Collection<T> for UInt{
    type Item = u64;
}
pub struct Test<T> {
    freq: Collection<T>::Item,
}

impl<Wrapper> Test<Wrapper> {
    pub fn new() -> Self {
        Self {
            freq: Collection::<Wrapper>::{},,
        }
    }
}

fn hoge<Wrapper: Collection>() {
    let t =  Wrapper::Member::<UInt>{}
}
