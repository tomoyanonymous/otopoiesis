use std::sync::Arc;

use super::{Generator, Region};
use script::parameter::FloatParameter;
use serde::{Deserialize, Serialize};

/// Data structure for track.
/// The track has some input/output streams.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Track {
    pub label: String,
    pub track_content: TrackContent,
    pub parameters: Vec<Arc<FloatParameter>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TrackContent {
    ///Contains Multiple Regions.
    Regions(Vec<Region>),
    ///Contains multiple sub-tracks.
    SubTracks(Vec<Box<Track>>),
    ///Contains one audio generator(0 input).
    Generator(Generator),
}

impl Track {
    pub fn new(name:&str) -> Self {
        Self {
            label: name.to_string(),
            track_content: TrackContent::Regions(vec![]),
            parameters: Vec::new(),
        }
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "track {}", self.label)
        write!(f, "track")
    }
}
