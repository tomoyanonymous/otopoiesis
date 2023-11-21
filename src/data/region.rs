use crate::script::{EvalError, Value};
use crate::{
    data::atomic,
    parameter::{FloatParameter, Parameter, RangedNumeric},
};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct FadeParam {
    pub time_in: Arc<FloatParameter>,
    pub time_out: Arc<FloatParameter>,
}
impl FadeParam {
    pub fn new() -> Self {
        Self {
            time_in: Arc::new(FloatParameter::new(0.0, "in_time").set_range(0.0..=1000.0)),
            time_out: Arc::new(FloatParameter::new(0.0, "out_time").set_range(0.0..=1000.0)),
        }
    }
    pub fn new_with(time_in: Arc<FloatParameter>, time_out: Arc<FloatParameter>) -> Self {
        Self { time_in, time_out }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ReplicateParam {
    pub count: atomic::U32,
}
impl From<u32> for ReplicateParam {
    fn from(v: u32) -> Self {
        Self { count: v.into() }
    }
}

/// Region filter transforms another region.
/// Maybe the region after transformation has different range from the origin.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RegionFilter {
    Gain,
    FadeInOut(FadeParam),
    Reverse,
    Replicate(ReplicateParam),
    Script(Value),
}
impl TryFrom<&Value> for RegionFilter {
    type Error = EvalError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Ok(Self::Script(value.clone()))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Content {
    Generator(Value),
    Transformer(RegionFilter, Box<Region>),
}

/// Data structure for region.
/// The region has certain start time and end time, and one generator (including an audio file).

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Region {
    /// start and dur stores a real time, not in sample.
    pub start: Arc<FloatParameter>,
    pub dur: Arc<FloatParameter>,
    pub content: Content,
    pub label: String,
}

impl Region {
    /// Utility function that converts a raw region into the region with fadein/out transformer.
    ///
    pub fn new(
        start: Arc<FloatParameter>,
        dur: Arc<FloatParameter>,
        content: Content,
        label: impl Into<String>,
    ) -> Self {
        Self {
            start,
            dur,
            content,
            label: label.into(),
        }
    }
    pub fn with_fade(origin: Self) -> Self {
        Self::new(
            origin.start.clone(),
            origin.dur.clone(),
            Content::Transformer(
                RegionFilter::FadeInOut(FadeParam::new()),
                Box::new(origin.clone()),
            ),
            origin.label,
        )
    }
    pub fn getrange(&self) -> RangeInclusive<f64> {
        let start = self.start.get() as f64;
        let end = start + self.dur.get() as f64;
        start..=end
    }
}

impl std::default::Default for Region {
    fn default() -> Self {
        Self {
            start: Arc::new(FloatParameter::default()),
            dur: Arc::new(FloatParameter::default()),
            content: Content::Generator(Value::None),
            label: "".to_string(),
        }
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "region {}", self.label)
    }
}
fn make_region_from_param(
    start: Arc<FloatParameter>,
    dur: Arc<FloatParameter>,
    content: &Value,
    label: &str,
) -> Result<Region, EvalError> {
    let content = Content::Generator(content.clone());
    let res = Region::new(start.clone(), dur.clone(), content, label);
    Ok(res)
}

impl TryFrom<&Value> for Region {
    type Error = EvalError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Region(env, start, dur, content, label, _) => {
                let start = start.eval(env.clone(), &None, &mut None)?;
                let dur = dur.eval(env.clone(), &None, &mut None)?;

                let content = content.eval(env.clone(), &None, &mut None)?;
                match (start, dur) {
                    (Value::Parameter(start), Value::Parameter(dur)) => {
                        make_region_from_param(start.clone(), dur.clone(), &content, label)
                    }
                    _ => Err(EvalError::InvalidConversion),
                }
            }
            _ => Err(EvalError::InvalidConversion),
        }
    }
}
