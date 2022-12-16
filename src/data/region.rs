use super::generator::*;
use crate::data::{atomic, AtomicRange};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioFile {
    file: String,
    length: usize,
    trim_range: AtomicRange,
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

/// Region filter transforms another region.
/// Maybe the region after transformation has different range from the origin.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RegionFilter {
    Gain,
    FadeInOut(Arc<FadeParam>),
    Reverse,
    Replicate(Arc<ReplicateParam>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Content {
    Generator(Arc<Generator>),
    AudioFile(AudioFile),
    Transformer(Arc<RegionFilter>, Arc<Region>),
}

/// Data structure for region.
/// The region has certain start time and end time, and one generator (including an audio file).

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Region {
    /// range stores a real time, not in sample.
    pub range: Arc<AtomicRange>,
    pub content: Content,
    pub label: String,
}

impl Region {
    /// Utility function that converts a raw region into the region with fadein/out transformer.
    ///
    pub fn new(range: AtomicRange, content: Content, label: impl Into<String>) -> Self {
        Self {
            range: Arc::new(range),
            content,
            label: label.into(),
        }
    }
    pub fn with_fade(origin: Arc<Self>) -> Arc<Self> {
        Arc::new(Self::new(
            AtomicRange::new(origin.range.start(), origin.range.end()),
            Content::Transformer(
                Arc::new(RegionFilter::FadeInOut(Arc::new(FadeParam {
                    time_in: 0.1.into(),
                    time_out: 0.1.into(),
                }))),
                Arc::clone(&origin),
            ),
            origin.label.clone(),
        ))
    }
}

impl std::default::Default for Region {
    fn default() -> Self {
        Self {
            range: Arc::new(AtomicRange::new(0, 0)),
            content: Content::Generator(Arc::new(Generator::default())),
            label: "".to_string(),
        }
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "region {}", self.label)
    }
}
