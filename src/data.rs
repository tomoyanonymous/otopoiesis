// mod meta;
// data format for project file. serialized to json with serde.
use crate::action;
use crate::parameter::Parameter;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use undo;

use crate::{parameter::FloatParameter, utils::AtomicRange};

// #[derive(Serialize, Deserialize, Clone)]
pub struct AppModel {
    pub transport: Arc<Transport>,
    pub global_setting: Arc<GlobalSetting>,
    pub project: Arc<Project>,
    pub history: undo::Record<action::Action>,
}

impl AppModel {
    pub fn new(
        transport: Arc<Transport>,
        global_setting: Arc<GlobalSetting>,
        project: Arc<Project>,
    ) -> Self {
        Self {
            transport,
            global_setting,
            project,
            history: undo::Record::new(),
        }
    }
    pub fn can_undo(&self) -> bool {
        let history = &self.history;
        history.can_undo()
    }
    pub fn undo(&mut self) {
        let history = &mut self.history;
        let _ = history.undo(&mut ()).unwrap();
    }
    pub fn can_redo(&self) -> bool {
        let history = &self.history;
        history.can_redo()
    }
    pub fn redo(&mut self) {
        let history = &mut self.history;
        let _ = history.redo(&mut ()).unwrap();
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Transport {
    pub is_playing: AtomicBool,
    pub time: Arc<AtomicU64>, //in sample
}
impl Transport {
    pub fn new() -> Self {
        Self {
            is_playing: AtomicBool::from(false),
            time: Arc::new(AtomicU64::from(0)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct GlobalSetting;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub sample_rate: AtomicU64,
    pub tracks: SharedVec<Track>,
}

pub type SharedVec<T> = Arc<Mutex<Vec<T>>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Track(pub SharedVec<Arc<Region>>);

impl Track {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(vec![])))
    }
}
impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "track {}", self.label)
        write!(f, "track")
    }
}
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

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "region {}", self.label)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OscillatorParam {
    pub amp: FloatParameter,
    pub freq: FloatParameter,
    pub phase: FloatParameter,
}
impl Default for OscillatorParam {
    fn default() -> Self {
        Self {
            amp: FloatParameter::new(1.0, 0.0..=1.0, "amp"),
            freq: FloatParameter::new(440.0, 20.0..=20000.0, "freq"),
            phase: FloatParameter::new(0.0, 0.0..=std::f32::consts::PI * 2.0, "phase"),
        }
    }
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
