//! Generic parameter data structure with a bounded range. Shared between GUI and Audio thread.

use mimium_lang::{
    ast::{Expr, Literal},
    interner::{ExprNodeId, ToSymbol},
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, ops::RangeInclusive, sync::Arc};

use crate::atomic::{self, SimpleAtomic};
pub trait Parameter: Clone + std::fmt::Debug {
    type Element;
    type Content;
    fn new(init: Self::Content, label: impl Into<String>) -> Self;
    fn get(&self) -> Self::Element;
    fn set(&self, v: Self::Element);
    fn get_label(&self) -> &str;
}
pub trait RangedNumeric {
    type Element;
    fn set_range(&mut self, r: RangeInclusive<Self::Element>) -> Self;
    fn get_range(&self) -> RangeInclusive<Self::Element>;
}
pub trait NumericParameter: Parameter + RangedNumeric {}

///Bool Parameter

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct BoolParameter {
//     value: atomic::Bool,
//     label: String,
// }
// impl Parameter for BoolParameter {
//     type Element = bool;

//     fn new(init: Self::Element, label: impl Into<String>) -> Self {
//         Self {
//             value: atomic::Bool::from(init),
//             label: label.into(),
//         }
//     }

//     fn get(&self) -> Self::Element {
//         self.value.load()
//     }

//     fn set(&self, v: Self::Element) {
//         self.value.store(v)
//     }

//     fn get_label(&self) -> &str {
//         &self.label
//     }
// }

///Float Parameter

#[derive(Clone, Debug)]
pub struct FloatParameter {
    //because the expression storage is thread-local, we can not share Expr refcell between audio thread and GUI.
    value: atomic::F64,
    pub range: RangeInclusive<atomic::F32>,
    label: String,
}

impl FloatParameter {
    pub fn get_value(&self) -> f64 {
        self.value.load()
    }
    pub fn set_value(&self, v: f64) {
        self.value.store(v);
    }
}

impl Parameter for FloatParameter {
    type Element = f32;
    type Content = atomic::F64;
    fn new(init: Self::Content, label: impl Into<String>) -> Self {
        Self {
            value: init,
            range: atomic::F32::from(f32::MIN)..=atomic::F32::from(f32::MAX),
            label: label.into(),
        }
    }

    fn get(&self) -> Self::Element {
        self.get_value() as f32
    }
    // note that no need to be "&mut self" here.
    fn set(&self, v: Self::Element) {
        self.set_value(v as f64);
    }

    fn get_label(&self) -> &str {
        &self.label
    }
}

impl RangedNumeric for FloatParameter {
    type Element = f32;
    fn get_range(&self) -> RangeInclusive<Self::Element> {
        let r = &self.range;
        r.start().load()..=r.end().load()
    }

    fn set_range(&mut self, r: RangeInclusive<Self::Element>) -> Self {
        self.range.start().store(*r.start());
        self.range.end().store(*r.end());
        self.clone()
    }
}
impl NumericParameter for FloatParameter {}

#[macro_export]
macro_rules! param_float {
    ($init:expr,$label:literal,$range:expr) => {
        FloatParameter::new($init, $label).set_range($range)
    };
}

// ///Integer Parameter

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct UIntParameter {
//     value: atomic::U64,
//     pub range: RangeInclusive<u64>,
//     label: String,
// }

// impl Parameter for UIntParameter {
//     type Element = u64;
//     fn new(init: Self::Element, label: impl Into<String>) -> Self {
//         Self {
//             value: atomic::U64::from(init),
//             range: u64::MIN..=u64::MAX,
//             label: label.into(),
//         }
//     }

//     fn get(&self) -> Self::Element {
//         self.value.load()
//     }
//     // note that no need to be "&mut self" here.
//     fn set(&self, v: Self::Element) {
//         self.value
//             .store(v.max(*self.range.start()).min(*self.range.end()));
//     }

//     fn get_label(&self) -> &str {
//         &self.label
//     }
// }
// impl RangedNumeric for UIntParameter {
//     type Element = u64;

//     fn get_range(&self) -> RangeInclusive<Self::Element> {
//         self.range.clone()
//     }

//     fn set_range(&mut self, r: RangeInclusive<Self::Element>) -> Self {
//         self.range = r;
//         self.clone()
//     }
// }
// impl NumericParameter for UIntParameter {}

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
