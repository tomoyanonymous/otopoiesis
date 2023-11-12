use super::{
    generator::*,
    script::{Expr, Value},
    ConversionError,
};
use crate::{
    data::{atomic, AtomicRange},
    utils::atomic::{Bool, F32},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct FadeParam {
    pub time_in: atomic::F32,
    pub time_out: atomic::F32,
}
impl FadeParam {
    pub fn new() -> Self {
        Self {
            time_in: 0.0.into(),
            time_out: 0.0.into(),
        }
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
    FadeInOut(Arc<FadeParam>),
    Reverse,
    Replicate(ReplicateParam),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Content {
    Generator(Generator),
    Transformer(RegionFilter, Box<Region>),
}
impl TryFrom<&Value> for Content {
    type Error = ConversionError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            //||{FadeInOut(Region) }
            Value::ExtFunction(fname) => match fname.as_str() {
                "sinewave" => {
                    let kind = OscillatorFun::SineWave;
                    let param = OscillatorParam::default();
                    Ok(Content::Generator(Generator::Oscillator(
                        kind,
                        Arc::new(param),
                    )))
                }
                "sawtooth" => {
                    let kind = OscillatorFun::SawTooth(Arc::new(Bool::new(true)));
                    let param = OscillatorParam::default();
                    Ok(Content::Generator(Generator::Oscillator(
                        kind,
                        Arc::new(param),
                    )))
                }
                "rect" => {
                    let kind = OscillatorFun::Rectanglular(Arc::new(F32::new(0.5)));
                    let param = OscillatorParam::default();
                    Ok(Content::Generator(Generator::Oscillator(
                        kind,
                        Arc::new(param),
                    )))
                }
                "triangular" => {
                    let kind = OscillatorFun::Triangular;
                    let param = OscillatorParam::default();
                    Ok(Content::Generator(Generator::Oscillator(
                        kind,
                        Arc::new(param),
                    )))
                }

                _ => Err(ConversionError {}),
            },
            _ => Err(ConversionError {}),
        }
    }
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
                RegionFilter::FadeInOut(Arc::new(FadeParam {
                    time_in: 0.1.into(),
                    time_out: 0.1.into(),
                })),
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
            content: Content::Generator(Generator::default()),
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
    let content: Result<Content, ConversionError> = Content::try_from(content);
    let res = Region::new(range, content?, label);
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
                id,
                box Expr::App(
                    box Expr::Literal(Value::ExtFunction(regionfilter)),
                    box Expr::Literal(region),
                ),
            ) => match regionfilter.as_str() {
                "fadeinout" => {
                    let rg = Region::try_from(region)?;
                    let range = rg.range.clone();
                    let label = rg.label.clone();
                    let param = Arc::new(FadeParam::new());
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
