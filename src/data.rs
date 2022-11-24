// data format for project file. serialized to json with serde.
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use crate::{parameter::FloatParameter, utils::AtomicRange};
#[derive(Serialize, Deserialize)]
pub struct GlobalSetting;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub global_setting: GlobalSetting,
    pub sample_rate: u64,
    pub tracks: Arc<Vec<Arc<Track>>>,
}

#[derive(Serialize, Deserialize)]
pub struct Track(pub Vec<Arc<Region>>);

//range stores a real time.
#[derive(Serialize, Deserialize)]
pub struct Region {
    pub range: AtomicRange,
    pub max_size: AtomicU64,
    pub generator: Arc<Generator>,
    pub filters: Vec<Arc<RegionFilter>>,
}

#[derive(Serialize, Deserialize)]
pub struct OscillatorParam {
    pub amp: FloatParameter,
    pub freq: FloatParameter,
    pub phase: FloatParameter,
}

#[derive(Serialize, Deserialize)]
pub enum Generator {
    Oscillator(Arc<OscillatorParam>),
}

#[derive(Serialize, Deserialize)]
pub enum RegionFilter {
    Gain,
    FadeInOut,
}
