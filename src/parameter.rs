use atomic_float::*;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, *};
use serde_with::{serde_as,SerializeAs,DeserializeAs};
use std::ops::RangeInclusive;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;


pub trait Parameter<T> {
    fn new(init: T, range: RangeInclusive<T>, label: impl Into<String>) -> Self;
    fn get(&self) -> T;
    fn set(&self, v: T);
}
struct AtomicF32Dummy(f32);
impl SerializeAs<AtomicF32> for AtomicF32Dummy {
    fn serialize_as<S>(value: &AtomicF32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {  
        serializer.serialize_f32(value.load(Ordering::Relaxed))
    }
}
struct Visitor;
impl<'de> serde::de::Visitor<'de> for Visitor{
    type Value = AtomicF32;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "f32")
    }
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where
            E: de::Error, {
        Ok(AtomicF32::from(v))
    }
}

impl<'de> DeserializeAs<'de, AtomicF32> for AtomicF32Dummy {
    fn deserialize_as<D>(deserializer: D) -> Result<AtomicF32, D::Error>
    where
        D: Deserializer<'de>,
    {  
        deserializer.deserialize_f32(Visitor{})
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct FloatParameter {
    #[serde_as(as = "AtomicF32Dummy")]
    value: AtomicF32,
    range: RangeInclusive<f32>,
    label: String,
}

// impl Serialize for AtomicF32 {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//          serializer.serialize_f32(self.load(Ordering::Relaxed))
//     }
// }
// impl Deserialize for FloatParameter {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//         where
//             D: serde::Deserializer<'de> {
// deserializer.deserialize_struct("FloatParameter",["value","range","label"], ||)
// let mut res = serializer.serialize_struct("FloatParameter", 3)?;
//         res.serialize_field("value", &self.get())?;
//         res.serialize_field("range", &self.range)?;
//         res.serialize_field("label", &self.label)?;
//         res.end()
//     }
// }

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
    label: String,
}

impl Parameter<u64> for UIntParameter {
    fn new(init: u64, range: RangeInclusive<u64>, label: impl Into<String>) -> Self {
        Self {
            value: AtomicU64::new(init),
            range,
            label: label.into(),
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
