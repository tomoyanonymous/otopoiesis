use atomic_float;
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, sync::atomic};

pub struct Primitive<P, A>
where
    P: Copy,
    A: From<P>,
{
    v: A,
    phantom_data: PhantomData<P>,
}

pub trait SimpleAtomic<T>
where
    T: Copy,
{
    const ORDER: atomic::Ordering = atomic::Ordering::Relaxed;
    type Ty;
    fn load(&self) -> T;
    fn store(&self, v: T);
}
impl<P, A> From<P> for Primitive<P, A>
where
    P: Copy,
    A: From<P>,
{
    fn from(v: P) -> Self {
        Self {
            v: A::from(v),
            phantom_data: PhantomData::<P> {},
        }
    }
}

macro_rules! impl_simple_atomic {
    ($name:ident,$p:ty,$ps:literal,$a:ty) => {
        impl SimpleAtomic<$p> for Primitive<$p, $a> {
            type Ty = Primitive<$p, $a>;
            fn load(&self) -> $p {
                self.v.load(Self::ORDER)
            }
            fn store(&self, v: $p) {
                self.v.store(v, Self::ORDER)
            }
        }
        impl Into<$p> for Primitive<$p, $a> {
            fn into(self) -> $p {
                self.load()
            }
        }
        impl Clone for Primitive<$p, $a> {
            fn clone(&self) -> Self {
                Self::from(self.load())
            }
        }
        #[derive(Clone, Serialize, Deserialize)]
        #[serde(from=$ps,into=$ps)]
        pub struct $name(pub Primitive<$p, $a>);

        impl From<$p> for $name {
            fn from(v: $p) -> Self {
                $name(Primitive::<$p, $a>::from(v))
            }
        }
        impl Into<$p> for $name{
            fn into(self)-> $p {
                self.0.load()
            }
        }
    };
}

pub fn make_simple_atomic<P, A>(v: P) -> Primitive<P, A>
where
    P: Copy,
    A: From<P>,
{
    Primitive::<P, A>::from(v)
}

impl_simple_atomic!(Bool, bool, "bool", atomic::AtomicBool);
impl_simple_atomic!(Usize, usize, "usize", atomic::AtomicUsize);
impl_simple_atomic!(I8, i8, "i8", atomic::AtomicI8);
impl_simple_atomic!(U8, u8, "u8", atomic::AtomicU8);
impl_simple_atomic!(I16, i16, "i16", atomic::AtomicI16);
impl_simple_atomic!(U16, u16, "u16", atomic::AtomicU16);
impl_simple_atomic!(I32, i32, "i32", atomic::AtomicI32);
impl_simple_atomic!(U32, u32, "u32", atomic::AtomicU32);
impl_simple_atomic!(F32, f32, "f32", atomic_float::AtomicF32);
impl_simple_atomic!(I64, i64, "i64", atomic::AtomicI64);
impl_simple_atomic!(U64, u64, "u64", atomic::AtomicU64);
impl_simple_atomic!(F64, f64, "f64", atomic_float::AtomicF64);


#[cfg(test)]
mod test {
    use super::*;
    use serde_json;
    #[test]
    fn boolean() {
        let t = Bool::from(true);
        let v: bool = t.into();
        assert_eq!(v, true);
    }
    #[test]
    fn serialize() {
        let t = U64::from(42);
        let json = serde_json::to_string_pretty(&t).unwrap();
        assert_eq!(json.to_string(), "42".to_string())
    }
}
