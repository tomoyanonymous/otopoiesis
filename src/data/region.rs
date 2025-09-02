use crate::{
    data::atomic,
    parameter::{FloatParameter, Parameter, RangedNumeric},
};

use mimium_lang::interner::ExprNodeId;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;
use std::sync::Arc;

// #[derive(Serialize, Deserialize, Clone, Default, Debug)]
// pub struct FadeParam {
//     pub time_in: Arc<FloatParameter>,
//     pub time_out: Arc<FloatParameter>,
// }
// impl FadeParam {
//     pub fn new() -> Self {
//         Self {
//             time_in: Arc::new(FloatParameter::new(0.0, "in_time").set_range(0.0..=1000.0)),
//             time_out: Arc::new(FloatParameter::new(0.0, "out_time").set_range(0.0..=1000.0)),
//         }
//     }
//     pub fn new_with(time_in: Arc<FloatParameter>, time_out: Arc<FloatParameter>) -> Self {
//         Self { time_in, time_out }
//     }
// }

// #[derive(Serialize, Deserialize, Clone, Default, Debug)]
// pub struct ReplicateParam {
//     pub count: atomic::U32,
// }
// impl From<u32> for ReplicateParam {
//     fn from(v: u32) -> Self {
//         Self { count: v.into() }
//     }
// }

// /// Region filter transforms another region.
// /// Maybe the region after transformation has different range from the origin.
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub enum RegionFilter {
//     Gain,
//     FadeInOut(FadeParam),
//     Reverse,
//     Replicate(ReplicateParam),
//     Script(Value),
// }
// impl TryFrom<&Value> for RegionFilter {
//     type Error = EvalError;

//     fn try_from(value: &Value) -> Result<Self, Self::Error> {
//         Ok(Self::Script(value.clone()))
//     }
// }

#[derive(Clone, Debug)]
pub enum RegionContent {
    Expr(Option<Vec<f64>>),
    // LiveExpr(ExprNodeId)
}

/// `Data structure for region.
/// The region has certain start time and end time, and one generator (including an audio file).

#[derive(Clone, Debug)]
pub struct Region {
    /// start and dur stores a real time, not in sample.
    pub start: Arc<FloatParameter>,
    pub dur: Arc<FloatParameter>,
    pub label: String,
    pub content: RegionContent,
    pub parameters: Vec<Arc<FloatParameter>>,

}

impl Region {
    /// Utility function that converts a raw region into the region with fadein/out transformer.
    ///
    pub fn new(
        start: Arc<FloatParameter>,
        dur: Arc<FloatParameter>,
        content: RegionContent,
        label: impl Into<String>,
        parameters: Vec<Arc<FloatParameter>>,
    ) -> Self {
        Self {
            start,
            dur,
            content,
            label: label.into(),
            parameters,
        }
    }
}

impl std::default::Default for Region {
    fn default() -> Self {
        Self {
            start: Arc::new(FloatParameter::default()),
            dur: Arc::new(FloatParameter::default()),
            content: RegionContent::Generator,
            label: "".to_string(),
            parameters: Vec::new(),
        }
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "region {}", self.label)
    }
}