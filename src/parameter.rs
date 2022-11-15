use atomic_float::*;
use std::ops::{Range, RangeInclusive};
use std::sync::atomic::Ordering;

pub trait Listener {
    fn on_value_change(&mut self, new_v: f32);
}
pub struct FloatParameter {
    value: AtomicF32,
    range: RangeInclusive<f32>,
    label: String,
}
//do we need?
unsafe impl Send for FloatParameter {}
unsafe impl Sync for FloatParameter {}

impl FloatParameter {
    pub fn new(init: f32, range: Range<f32>, label: impl Into<String>) -> Self {
        Self {
            value: AtomicF32::new(init),
            range: RangeInclusive::new(range.start, range.end),
            label: label.into(),
        }
    }

    pub fn get(&self) -> f32 {
        self.value.load(Ordering::Relaxed)
    }
    // note that no need to be "&mut self" here.
    pub fn set(&self, v: f32) {
        self.value.store(
            v.max(*self.range.start()).min(*self.range.end()),
            Ordering::Relaxed,
        );
    }
}
