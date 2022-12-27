//! The main data format like project file, track, region and etc. Can be (de)serialized to/from json with serde.

use crate::action;
use crate::utils::{atomic, AtomicRange, SimpleAtomic};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::sync::Arc;
use undo;

pub mod generator;
pub mod region;

pub use generator::*;
pub use region::*;

// #[derive(Serialize, Deserialize, Clone)]
pub struct AppModel {
    pub transport: Arc<Transport>,
    pub global_setting: GlobalSetting,
    pub project: Project,
    pub history: undo::Record<action::Action>,
}

impl AppModel {
    pub fn new(transport: Transport, global_setting: GlobalSetting, project: Project) -> Self {
        let transport = Arc::new(transport);
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
        let _ = history.undo(&mut self.project).unwrap();
    }
    pub fn can_redo(&self) -> bool {
        let history = &self.history;
        history.can_redo()
    }
    pub fn redo(&mut self) {
        let history = &mut self.history;
        let _ = history.redo(&mut self.project).unwrap();
    }
    pub fn get_track_for_id(&mut self, id: usize) -> Option<&mut Track> {
        self.project.tracks.get_mut(id)
    }
}

pub enum PlayOp {
    Play = 0,
    Pause = 1,
    Halt = 2,
}

impl From<u8> for PlayOp {
    fn from(p: u8) -> Self {
        match p {
            0 => Self::Play,
            1 => Self::Pause,
            2 => Self::Halt,
            _ => panic!("invalid operation"),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Transport {
    is_playing: atomic::U8,
    pub time: Arc<atomic::U64>, //in sample
    playing_history: atomic::U8,
}

impl Transport {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn request_play(&self, p: PlayOp) {
        self.playing_history.store(self.is_playing.load());
        self.is_playing.store(p as u8);
    }
    pub fn is_playing(&self) -> bool {
        match PlayOp::from(self.is_playing.load()) {
            PlayOp::Play => true,
            PlayOp::Pause | PlayOp::Halt => false,
        }
    }
    pub fn ready_to_trigger(&self) -> Option<PlayOp> {
        if self.is_playing.load() != self.playing_history.load() {
            let res = Some(PlayOp::from(self.is_playing.load()));
            self.playing_history.store(self.is_playing.load() as u8);
            res
        } else {
            None
        }
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            is_playing: atomic::U8::from(2),
            time: Arc::new(atomic::U64::from(0)),
            playing_history: atomic::U8::from(2),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct GlobalSetting;

/// A main project data. It should be imported/exported via serde.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub sample_rate: atomic::U64,
    pub tracks: Vec<Track>,
}

/// Data structure for track.
/// The track has some input/output stream.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Track {
    ///Contains Multiple Regions.
    /// TODO:Change container for this to be HashedSet for the more efficient implmentation of Undo Action.
    Regions(Vec<Region>),
    ///Contains one audio generator(0 input).
    Generator(Generator),
    ///Take another track and transform it (like filter).
    Transformer(),
}

impl Track {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Track {
    fn default() -> Self {
        Track::Regions(vec![])
    }
}
impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "track {}", self.label)
        write!(f, "track")
    }
}
