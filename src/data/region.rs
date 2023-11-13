use super::{
    script::{Expr, Value},
    ConversionError,
};
use crate::{
    data::{atomic, AtomicRange},
    parameter::{FloatParameter, Parameter, RangedNumeric},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct FadeParam {
    pub time_in: Arc<FloatParameter>,
    pub time_out: Arc<FloatParameter>,
}
impl FadeParam {
    pub fn new() -> Self {
        Self {
            time_in: Arc::new(FloatParameter::new(0.0, "in_time").set_range( 0.0..=1000.0)),
            time_out: Arc::new(FloatParameter::new(0.0, "out_time").set_range( 0.0..=1000.0)),
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
    /// range stores a real time, not in sample.
    pub range: AtomicRange<f64>,
    pub content: Content,
    pub label: String,
}

impl Region {
    /// Utility function that converts a raw region into the region with fadein/out transformer.
    ///
    pub fn new(range: AtomicRange<f64>, content: Content, label: impl Into<String>) -> Self {
        Self {
            range,
            content,
            label: label.into(),
        }
    }
    pub fn with_fade(origin: Self) -> Self {
        Self::new(
            AtomicRange::<f64>::new(origin.range.start(), origin.range.end()),
            Content::Transformer(
                RegionFilter::FadeInOut(FadeParam::new()),
                Box::new(origin.clone()),
            ),
            origin.label,
        )
    }
}

impl std::default::Default for Region {
    fn default() -> Self {
        Self {
            range: AtomicRange::<f64>::new(0.0, 0.0),
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
    start: f64,
    dur: f64,
    content: &Value,
    label: &str,
) -> Result<Region, ConversionError> {
    let range = AtomicRange::new(start, start + dur);
    let content = Content::Generator(content.clone());
    let res = Region::new(range, content, label);
    Ok(res)
}

impl TryFrom<&Value> for Region {
    type Error = ConversionError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Region(start, dur, content, label, _) => {
                make_region_from_param(*start, *dur, content, label)
            }
            Value::Function(
                _ids,
                box Expr::App(box Expr::Literal(Value::ExtFunction(regionfilter)), args),
            ) if args.len() == 3 => match (
                regionfilter.as_str(),
                args.get(0).unwrap(),
                args.get(1).unwrap(),
                args.get(2).unwrap(),
            ) {
                (
                    "fadeinout",
                    Expr::Literal(region),
                    Expr::Literal(Value::Parameter(time_in)),
                    Expr::Literal(Value::Parameter(time_out)),
                ) => {
                    //todo:need to eval region for non-literal expression
                    let rg = Region::try_from(region)?;
                    let range = rg.range.clone();
                    let label = rg.label.clone();
                    let param = FadeParam::new_with(time_in.clone(),time_out.clone());
                    let content =
                        Content::Transformer(RegionFilter::FadeInOut(param), Box::new(rg));
                    Ok(Region::new(range, content, label))
                }
                _ => Err(ConversionError {}),
            },
            _ => Err(ConversionError {}),
        }
    }
}
