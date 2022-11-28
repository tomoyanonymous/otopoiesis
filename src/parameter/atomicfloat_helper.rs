use atomic_float::*;

use serde::*;
use serde_with::{DeserializeAs, SerializeAs};
use std::sync::atomic::Ordering;
pub struct AtomicF32Json(f32);
impl SerializeAs<AtomicF32> for AtomicF32Json {
    fn serialize_as<S>(value: &AtomicF32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(value.load(Ordering::Relaxed))
    }
}
struct Visitor;
impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = AtomicF32;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "floating point number")
    }
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(AtomicF32::from(v as f32))
    }
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(AtomicF32::from(v))
    }
}

impl<'de> DeserializeAs<'de, AtomicF32> for AtomicF32Json {
    fn deserialize_as<D>(deserializer: D) -> Result<AtomicF32, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_f32(Visitor {})
    }
}
