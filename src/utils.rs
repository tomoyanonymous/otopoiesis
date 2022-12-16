//! Misc utilities such as Atomic Structure.

pub mod atomic;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct AtomicRange(pub Arc<atomic::U64>, pub Arc<atomic::U64>);

type EguiGetSet<'a> = Box<dyn FnMut(Option<f64>) -> f64 + 'a>;

impl AtomicRange {
    pub fn new(start: u64, end: u64) -> Self {
        Self(
            Arc::new(atomic::U64::from(start)),
            Arc::new(atomic::U64::from(end)),
        )
    }

    pub fn get_pair(&self) -> (u64, u64) {
        (self.start(), self.end())
    }
    pub fn start(&self) -> u64 {
        self.0.load()
    }
    pub fn end(&self) -> u64 {
        self.1.load()
    }
    pub fn getrange(&self) -> u64 {
        self.1.load() - self.0.load()
    }
    pub fn contains(&self, v: u64) -> bool {
        let (min, max) = self.get_pair();
        (min..max).contains(&v)
    }
    pub fn set_start(&self, v: u64) {
        self.0.store(v);
    }
    pub fn set_end(&self, v: u64) {
        self.1.store(v);
    }
    pub fn shift(&self, v: i64) {
        self.set_start((self.start() as i64 + v).max(0) as u64);
        self.set_end((self.end() as i64 + v).max(0) as u64);
    }
    //does not shrink when the range reached to 0.
    pub fn shift_bounded(&self, v: i64) {
        let start_bounded = (self.start() as i64 + v).max(0) as u64;
        let end_bounded = start_bounded + self.getrange();
        self.set_start(start_bounded);
        self.set_end(end_bounded);
    }
    pub fn egui_get_set_start(&self, scaling_factor: f64) -> EguiGetSet {
        Box::new(move |f: Option<f64>| -> f64 {
            if let Some(v) = f {
                self.set_start((v * scaling_factor) as u64);
            }
            self.get_pair().0 as f64 / scaling_factor
        })
    }
    pub fn egui_get_set_end(&self, scaling_factor: f64) -> EguiGetSet {
        Box::new(move |f: Option<f64>| -> f64 {
            if let Some(v) = f {
                self.set_end((v * scaling_factor) as u64);
            }
            self.get_pair().1 as f64 / scaling_factor
        })
    }
}
impl Clone for AtomicRange {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0), Arc::clone(&self.1))
    }
}
impl From<std::ops::Range<u64>> for AtomicRange {
    fn from(t: std::ops::Range<u64>) -> Self {
        Self::new(t.start, t.end)
    }
}

impl From<std::ops::RangeInclusive<u64>> for AtomicRange {
    fn from(t: std::ops::RangeInclusive<u64>) -> Self {
        Self::new(*t.start(), *t.end())
    }
}
impl From<&AtomicRange> for std::ops::RangeInclusive<u64> {
    fn from(t: &AtomicRange) -> Self {
        t.start()..=t.end()
    }
}
