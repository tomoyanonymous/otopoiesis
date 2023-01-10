use super::generator::*;
use crate::data::{atomic, AtomicRange};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioFile {
    file: String,
    length: usize,
    trim_range: AtomicRange<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct FadeParam {
    pub time_in: atomic::F32,
    pub time_out: atomic::F32,
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
    AudioFile(AudioFile),
    Transformer(RegionFilter, Box<Region>),
    // Array(Vec<Arc<Region>>),
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
    // pub fn interpret(&self) -> Self {
    //     match &self.content {
    //         Content::Transformer(p, origin) if matches!(p, RegionFilter::Replicate(_)) => {
    //             if let RegionFilter::Replicate(param) = p {
    //                 let mut range = origin.range.clone();
    //                 let mut len = range.getrange();
    //                 let mut last = 0;
    //                 todo!()
    //                 // let arr = Content::Array(
    //                 //     (0..param.count.load())
    //                 //         .into_iter()
    //                 //         .map(|_| {
    //                 //             //とりあえず位置をずらして複製
    //                 //             range = Arc::new(AtomicRange<i64>::new(last, last + len));
    //                 //             len = range.getrange();
    //                 //             last = range.end();
    //                 //             range.clone()
    //                 //         })
    //                 //         .enumerate()
    //                 //         .map(|(i, newrange)| {
    //                 //             Arc::new(Region::new(
    //                 //                 newrange.as_ref().clone(),
    //                 //                 origin.content.clone(),
    //                 //                 format!("{}_rep_{}", origin.label, i),
    //                 //             ))
    //                 //         })
    //                 //         .collect(),
    //                 // );
    //                 Self::new(
    //                     AtomicRange<i64>::new(origin.range.start(), last),
    //                     arr,
    //                     format!("{}_arr", origin.label),
    //                 )
    //             } else {
    //                 unreachable!()
    //             }
    //         }
    //         _ => self.clone(),
    //     }
    // }
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
