//! The main data format like project file, track, region and etc. Can be (de)serialized to/from json with serde.

use crate::action;
use crate::app::filemanager::{self, FileManager};
use crate::utils::{atomic, AtomicRange, SimpleAtomic};

use rfd;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use undo;

pub mod script;
pub mod generator;
pub mod region;
pub mod track;

pub use generator::*;
pub use region::*;
pub use track::*;

#[cfg(not(feature = "web"))]
use dirs;

pub struct LaunchArg {
    pub file: Option<String>,
    pub project_root: Option<String>,
    pub config_dir: Option<String>,
    pub log_level: u8,
}
impl Default for LaunchArg {
    fn default() -> Self {
        #[cfg(not(feature = "web"))]
        let config_dir = dirs::home_dir().map(|mut p| {
            p.push(std::path::PathBuf::from(".otopoiesis"));
            p.to_str().unwrap_or("").to_string()
        });
        #[cfg(feature = "web")]
        let config_dir = None;
        Self {
            file: None,
            project_root: None,
            config_dir,
            log_level: 3,
        }
    }
}

// #[derive(Serialize, Deserialize, Clone)]
pub struct AppModel {
    pub transport: Arc<Transport>,
    pub global_setting: GlobalSetting,
    pub launch_arg: LaunchArg,
    pub project: Project,
    pub project_str: String,
    pub project_file: Option<String>,
    pub history: undo::Record<action::Action>,
    pub action_tx: mpsc::Sender<action::Action>,
    pub action_rx: mpsc::Receiver<action::Action>,
    pub builtin_fns:HashMap<&'static str,script::ExtFun>
}

impl AppModel {
    pub fn new(transport: Transport, global_setting: GlobalSetting, launch_arg: LaunchArg) -> Self {
        let transport = Arc::new(transport);
        let file = launch_arg.file.clone();
        let project_file = file.map(|file| {
            let path = std::path::PathBuf::from(file);
            String::from(path.to_string_lossy())
        });
        let mut project_str = String::new();
        if let Some(file) = project_file.clone() {
            let _ = filemanager::get_global_file_manager().read_to_string(file, &mut project_str);
        }
        let (action_tx, action_rx) = mpsc::channel();
        Self {
            transport,
            global_setting,
            launch_arg,
            project: Project::new(44100),
            project_str,
            project_file,
            history: undo::Record::new(),
            action_tx,
            action_rx,
            builtin_fns:script::builtin_fn::gen_default_functions()
        }
    }
    pub fn get_builtin_fn(&self,name:&str)->Option<&script::ExtFun>{
        self.builtin_fns.get(name)
    }
    pub fn can_undo(&self) -> bool {
        let history = &self.history;
        history.can_undo()
    }

    pub fn undo(&mut self) {
        let history = &mut self.history;
        if let Some(Err(e)) = history.undo(&mut self.project) {
            eprintln!("{}", e)
        }
    }
    pub fn can_redo(&self) -> bool {
        let history = &self.history;
        history.can_redo()
    }
    pub fn redo(&mut self) {
        let history = &mut self.history;
        let _ = history.redo(&mut self.project).unwrap();
    }

    pub fn open_file(&mut self) {
        #[cfg(not(feature = "web"))]
        {
            let dir = self.project_file.clone().unwrap_or("~/".to_string());
            let file = rfd::FileDialog::new()
                .add_filter("json", &["json"])
                .set_directory(dir)
                .pick_file();
            let path_str = String::from(file.unwrap().to_string_lossy());

            let _ =
                filemanager::GLOBAL_FILE_MANAGER.read_to_string(path_str, &mut self.project_str);
        }
    }
    pub fn save_file(&mut self) {
        match &self.project_file {
            Some(file) => {
                let _ = filemanager::GLOBAL_FILE_MANAGER
                    .save_file(file.clone(), self.project_str.clone());
            }
            None => {
                self.save_as_file();
            }
        }
    }
    pub fn save_as_file(&mut self) {
        #[cfg(not(feature = "web"))]
        {
            let dir = self.project_file.clone().unwrap_or("~/".to_string());
            let file = rfd::FileDialog::new()
                .set_directory(dir)
                .add_filter("json", &["json"])
                .save_file();
            let path_str = String::from(file.unwrap().to_string_lossy());
            let _ = filemanager::GLOBAL_FILE_MANAGER
                .save_file(path_str.clone(), self.project_str.clone());
            self.project_file = Some(path_str);
        }
    }
    pub fn ui_to_code(&mut self) {
        let json = serde_json::to_string_pretty(&self.project);
        let json_str = json.unwrap_or_else(|e| {
            println!("{}", e);
            "failed to print".to_string()
        });
        self.project_str = json_str;
    }
    pub fn code_to_ui(&mut self) -> Result<(), serde_json::Error> {
        serde_json::from_str::<Project>(&self.project_str).map(|proj| {
            self.project = proj.clone();
        })
    }
    pub fn get_track_for_id_mut(&mut self, id: usize) -> Option<&mut Track> {
        self.project.tracks.get_mut(id)
    }
    pub fn get_track_for_id(&self, id: usize) -> Option<&Track> {
        self.project.tracks.get(id)
    }
    pub fn consume_actions(&mut self) -> bool {
        let mut ui_need_update = false;
        for action_received in self.action_rx.try_iter() {
            let _res = self.history.apply(&mut self.project, action_received);
            ui_need_update = true;
        }
        ui_need_update
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
            self.playing_history.store(self.is_playing.load());
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
impl Project {
    fn new(sample_rate: u64) -> Self {
        Self {
            sample_rate: atomic::U64::from(sample_rate),
            tracks: vec![],
        }
    }
}
