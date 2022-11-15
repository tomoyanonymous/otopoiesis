use std::ops::RangeInclusive;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
pub struct AtomicRange(pub AtomicU64, pub AtomicU64);

impl AtomicRange {
    pub fn new(start: u64, end: u64) -> Self {
        Self(AtomicU64::from(start), AtomicU64::from(end))
    }
    pub fn get_pair(&self) -> (u64, u64) {
        (self.start(), self.end())
    }
    pub fn start(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
    pub fn end(&self) -> u64 {
        self.1.load(Ordering::Relaxed)
    }
    pub fn getrange(&self) -> u64 {
        &self.1.load(Ordering::Relaxed) - &self.0.load(Ordering::Relaxed)
    }
    pub fn contains(&self, v: &u64) -> bool {
        let (min, max) = self.get_pair();
        RangeInclusive::<u64>::new(min, max).contains(v)
    }
}
