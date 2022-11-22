// data format for project file. serialized to json with serde.
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::utils::AtomicRange;
#[derive(Serialize, Deserialize)]
pub struct GlobalSetting;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub global_setting: GlobalSetting,
    pub tracks: Vec<Track>,
}

#[derive(Serialize, Deserialize)]
pub struct Track(Vec<Region>);

//range stores a real time.
#[derive(Serialize, Deserialize)]
pub struct Region {
    pub range: std::ops::Range<f32>,
    pub generator: Generator,
    pub filters: Vec<RegionFilter>,
}

#[derive(Serialize, Deserialize)]
pub enum Generator {
    Sinewave,
}

#[derive(Serialize, Deserialize)]
pub enum RegionFilter {
    Gain,
    FadeInOut,
}
