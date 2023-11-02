//! Generic parameter data structure with a bounded range. Shared between GUI and Audio thread.

use crate::utils::atomic::{self, SimpleAtomic};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

pub trait Parameter<T> {
    fn new(init: T, range: RangeInclusive<T>, label: impl Into<String>) -> Self;
    fn get(&self) -> T;
    fn set(&self, v: T);
    fn get_range(&self) -> &RangeInclusive<T>;
    fn get_label(&self) -> &str;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FloatParameter {
    value: atomic::F32,
    pub range: RangeInclusive<f32>,
    label: String,
}
//do we need?

impl Parameter<f32> for FloatParameter {
    fn new(init: f32, range: RangeInclusive<f32>, label: impl Into<String>) -> Self {
        Self {
            value: atomic::F32::from(init),
            range,
            label: label.into(),
        }
    }

    fn get(&self) -> f32 {
        self.value.clone().into()
    }
    // note that no need to be "&mut self" here.
    fn set(&self, v: f32) {
        self.value
            .store(v.max(*self.range.start()).min(*self.range.end()));
    }

    fn get_label(&self) -> &str {
        &self.label
    }
    fn get_range(&self) -> &RangeInclusive<f32> {
        &self.range
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UIntParameter {
    value: atomic::U64,
    pub range: RangeInclusive<u64>,
    label: String,
}

impl Parameter<u64> for UIntParameter {
    fn new(init: u64, range: RangeInclusive<u64>, label: impl Into<String>) -> Self {
        Self {
            value: atomic::U64::from(init),
            range,
            label: label.into(),
        }
    }

    fn get(&self) -> u64 {
        self.value.load()
    }
    // note that no need to be "&mut self" here.
    fn set(&self, v: u64) {
        self.value
            .store(v.max(*self.range.start()).min(*self.range.end()));
    }

    fn get_label(&self) -> &str {
        &self.label
    }
    fn get_range(&self) -> &RangeInclusive<u64> {
        &self.range
    }
}

impl Default for FloatParameter {
    fn default() -> Self {
        Self::new(0., 0.0..=f32::MAX, "")
    }
}
// pub struct UIntPairParameter {
//     value: (AtomicU64,AtomicInt),
//     range: RangeInclusive<u64,u64>,
//     label: String,
// }

// impl Parameter<u64> for UIntParameter {
//     fn new(init: u64, range: RangeInclusive<u64>, label: impl Into<String>) -> Self {
//         Self {
//             value: AtomicU64::new(init),
//             range,
//             label: label.into(),
//         }
//     }

//     fn get(&self) -> u64 {
//         self.value.load(Ordering::Relaxed)
//     }
//     // note that no need to be "&mut self" here.
//     fn set(&self, v: u64) {
//         self.value.store(
//             v.max(*self.range.start()).min(*self.range.end()),
//             Ordering::Relaxed,
//         );
//     }
// }
