use atomic_float::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::ops::RangeInclusive;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

mod atomicfloat_helper;
pub use atomicfloat_helper::AtomicF32Json;
pub use atomicfloat_helper::AtomicF64Json;

pub trait Parameter<T> {
    fn new(init: T, range: RangeInclusive<T>, label: impl Into<String>) -> Self;
    fn get(&self) -> T;
    fn set(&self, v: T);
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct FloatParameter {
    #[serde_as(as = "AtomicF32Json")]
    value: AtomicF32,
    pub range: RangeInclusive<f32>,
    label: String,
}
//do we need?
// unsafe impl Send for FloatParameter {}
// unsafe impl Sync for FloatParameter {}

impl Parameter<f32> for FloatParameter {
    fn new(init: f32, range: RangeInclusive<f32>, label: impl Into<String>) -> Self {
        Self {
            value: AtomicF32::new(init),
            range,
            label: label.into(),
        }
    }

    fn get(&self) -> f32 {
        self.value.load(Ordering::Relaxed)
    }
    // note that no need to be "&mut self" here.
    fn set(&self, v: f32) {
        self.value.store(
            v.max(*self.range.start()).min(*self.range.end()),
            Ordering::Relaxed,
        );
    }
}


pub struct UIntParameter {
    value: AtomicU64,
    range: RangeInclusive<u64>,
    _label: String,
}

impl Parameter<u64> for UIntParameter {
    fn new(init: u64, range: RangeInclusive<u64>, label: impl Into<String>) -> Self {
        Self {
            value: AtomicU64::new(init),
            range,
            _label: label.into(),
        }
    }

    fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
    // note that no need to be "&mut self" here.
    fn set(&self, v: u64) {
        self.value.store(
            v.max(*self.range.start()).min(*self.range.end()),
            Ordering::Relaxed,
        );
    }
}

impl Clone for FloatParameter {
    fn clone(&self) -> Self {
        Self::new(self.get(), self.range.clone(), self.label.clone())
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
