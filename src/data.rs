// mod meta;
// data format for project file. serialized to json with serde.
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::{parameter::FloatParameter, utils::AtomicRange};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct GlobalSetting;

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub global_setting: GlobalSetting,
    pub sample_rate: u64,
    pub tracks: SharedParamsRt<Vec<Track>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "T")]
pub struct SharedParamsRt<T>
where
    T: Clone + Deref,
{
    // pub local: T,
    pub shared: Arc<Mutex<T>>,
}
impl<T> SharedParamsRt<T>
where
    T: Clone + Deref,
{
    pub fn new(t: T) -> Self {
        Self {
            // local: *local_lock,
            shared: Arc::new(Mutex::new(t)),
        }
    }
    pub fn from(t: Arc<Mutex<T>>) -> Self {
        Self {
            shared: Arc::clone(&t),
        }
    }
    // pub fn sync_rt(&mut self) {
    //     if let Ok(ref mut lock) = self.shared.try_lock() {

    //         self.local.clone_from(&*lock);
    //     }
    // }
    pub fn get_arc(&self) -> Arc<Mutex<T>> {
        self.shared.clone()
    }
}
impl<T> From<T> for SharedParamsRt<T>
where
    T: Clone + Deref,
{
    fn from(t: T) -> Self {
        Self {
            // local: t.clone(),
            shared: Arc::new(Mutex::new(t.clone())),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Track(pub SharedParamsRt<Vec<Arc<Region>>>);

pub type TrackShared = Arc<Mutex<Vec<Arc<Region>>>>;

//range stores a real time.
#[derive(Serialize, Deserialize)]
pub struct Region {
    pub range: AtomicRange,
    pub max_size: AtomicU64,
    pub generator: Arc<Generator>,
    pub filters: Vec<Arc<RegionFilter>>,
    pub label: String,
}

impl Clone for Region {
    fn clone(&self) -> Self {
        let max = self.max_size.load(Ordering::Relaxed);
        Self {
            range: self.range.clone(),
            max_size: AtomicU64::new(max),
            generator: self.generator.clone(),
            filters: self.filters.clone(),
            label: self.label.clone(),
        }
    }
}
impl std::default::Default for Region {
    fn default() -> Self {
        Self {
            range: AtomicRange::new(0, 0),
            max_size: AtomicU64::from(0),
            generator: Arc::new(Generator::Oscillator(Arc::new(OscillatorParam::default()))),
            filters: vec![],
            label: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct OscillatorParam {
    pub amp: FloatParameter,
    pub freq: FloatParameter,
    pub phase: FloatParameter,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Generator {
    Oscillator(Arc<OscillatorParam>),
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum RegionFilter {
    Gain,
    FadeInOut,
}
