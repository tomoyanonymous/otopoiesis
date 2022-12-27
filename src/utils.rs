//! Misc utilities such as Atomic Structure.
pub mod atomic;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

pub use self::atomic::{make_simple_atomic, SimpleAtomic, SimpleAtomicTest};
use atomic::IsAtomicNumber;

#[derive(Serialize, Deserialize, Debug)]
pub struct AtomicRange<T>(pub Arc<T::Composed>, pub Arc<T::Composed>)
where
    T: IsAtomicNumber<T> + std::cmp::Ord;

impl<T> AtomicRange<T>
where
    T: IsAtomicNumber<T> + std::cmp::Ord,
{
    pub fn new(start: T, end: T) -> Self {
        Self(
            Arc::new(make_simple_atomic(start)),
            Arc::new(make_simple_atomic(end)),
        )
    }

    pub fn get_pair(&self) -> (T, T) {
        (self.start(), self.end())
    }
    pub fn start(&self) -> T {
        self.0.load()
    }
    pub fn end(&self) -> T {
        self.1.load()
    }
    pub fn getrange(&self) -> T {
        self.1.load() - self.0.load()
    }
    pub fn contains(&self, v: T) -> bool {
        let (min, max) = self.get_pair();
        (min..max).contains(&v)
    }
    pub fn set_start(&self, v: T) {
        self.0.store(v);
    }
    pub fn set_end(&self, v: T) {
        self.1.store(v);
    }
    pub fn shift(&self, v: T) {
        self.set_start((self.start() + v).max(T::default()));
        self.set_end((self.end() + v).max(T::default()));
    }
    //does not shrink when the range reached to 0.
    pub fn shift_bounded(&self, v: T) {
        let start_bounded = (self.start() as T + v).max(T::default());
        let end_bounded = start_bounded + self.getrange();
        self.set_start(start_bounded);
        self.set_end(end_bounded);
    }
}
impl<T> Clone for AtomicRange<T>
where
    T: IsAtomicNumber<T> + std::cmp::Ord,
{
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0), Arc::clone(&self.1))
    }
}
impl<T> From<std::ops::Range<T>> for AtomicRange<T>
where
    T: IsAtomicNumber<T> + std::cmp::Ord,
{
    fn from(t: std::ops::Range<T>) -> Self {
        Self::new(t.start, t.end)
    }
}

impl<T> From<std::ops::RangeInclusive<T>> for AtomicRange<T>
where
    T: IsAtomicNumber<T> + std::cmp::Ord,
{
    fn from(t: std::ops::RangeInclusive<T>) -> Self {
        Self::new(*t.start(), *t.end())
    }
}
impl<T> From<&AtomicRange<T>> for std::ops::RangeInclusive<T>
where
    T: IsAtomicNumber<T> + std::cmp::Ord,
{
    fn from(t: &AtomicRange<T>) -> Self {
        t.start()..=t.end()
    }
}
