//! The main data format like project file, track, region and etc. Can be (de)serialized to/from json with serde.

use crate::app::filemanager::{self, FileManager};
use crate::atomic::{self, SimpleAtomic};
use crate::audio::renderer::PlayState;
use crate::{data, mimium_fns};

use crate::parameter::FloatParameter;
use coreaudio_sys::erfcf;
use mimium_lang::Config;
use mimium_lang::interner::ToSymbol;
use mimium_lang::runtime::vm::Machine;
use mimium_lang::{ExecContext, plugin};
use rfd;
use ringbuf::HeapCons;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use slotmap::{SlotMap, new_key_type};

use std::cell::{OnceCell, RefCell};
use std::io::BufWriter;
use std::rc::Rc;
use std::sync::{Arc, mpsc};
use undo;

// pub mod generator;
pub mod region;
pub mod track;

// pub use generator::*;
pub use region::*;
pub use track::*;

#[cfg(not(target_arch = "wasm32"))]
use dirs;

// #[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq, Hash)]
// pub struct ProbeId(usize);
new_key_type! {
    pub struct ProbeId;

}
pub type SliderId = usize;
pub type ProbeMap = SlotMap<ProbeId, HeapCons<f64>>;
pub type SliderMap = Vec<Arc<FloatParameter>>;

pub struct LaunchArg {
    pub file: Option<String>,
    pub project_root: Option<String>,
    pub config_dir: Option<String>,
    pub log_level: u8,
}
impl Default for LaunchArg {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let config_dir = dirs::home_dir().map(|mut p| {
            p.push(std::path::PathBuf::from(".otopoiesis"));
            p.to_str().unwrap_or("").to_string()
        });
        #[cfg(target_arch = "wasm32")]
        let config_dir = None;
        Self {
            file: None,
            project_root: None,
            config_dir,
            log_level: 3,
        }
    }
}
#[derive(Debug)]
pub struct ConversionError {}

// #[derive(Serialize, Deserialize, Clone)]
pub struct AppModel {
    pub playop_queue: mpsc::Sender<data::PlayOp>,
    pub playstate: PlayState,
    pub global_setting: GlobalSetting,
    pub launch_arg: LaunchArg,
    pub mimium_ctx: Option<ExecContext>,
    project_tx: mpsc::Sender<data::Project>,
    project_rx: mpsc::Receiver<data::Project>,
    pub project: Project,
    pub project_str: String,
    pub project_mir_str: String,
    pub bytecode_str: String,
    pub project_file: Option<String>,
    // pub history: undo::Record<action::Action>,
    // pub action_tx: mpsc::Sender<action::Action>,
    // pub action_rx: mpsc::Receiver<action::Action>,
}

impl AppModel {
    pub fn new(
        playop_queue: mpsc::Sender<data::PlayOp>,
        playstate: PlayState,
        global_setting: GlobalSetting,
        launch_arg: LaunchArg,
    ) -> Self {
        // let transport = Arc::new(transport);
        let file = launch_arg.file.clone();
        let project_file = file.map(|file| {
            let path = std::path::PathBuf::from(file);
            String::from(path.to_string_lossy())
        });
        let mut project_str = String::new();
        if let Some(file) = project_file.clone() {
            let _ = filemanager::get_global_file_manager().read_to_string(file, &mut project_str);
        }
        let (project_tx, project_rx) = mpsc::channel();
        Self {
            playop_queue,
            global_setting,
            playstate,
            launch_arg,
            mimium_ctx: None,
            project_tx,
            project_rx,
            project: Project::new(String::new(), 44100),
            project_str,
            project_mir_str: String::new(),
            bytecode_str: String::new(),
            project_file,
            // history: undo::Record::new(),
            // action_tx,
            // action_rx,
        }
    }
    pub fn can_undo(&self) -> bool {
        // let history = &self.history;
        // history.can_undo()
        false
    }

    pub fn undo(&mut self) {
        // let history = &mut self.history;
        // if let Some(Err(e)) = history.undo(&mut self.project_str) {
        //     eprintln!("{}", e)
        // };

        self.compile(self.project_str.clone().as_str());
        self.ui_to_code();
    }
    pub fn can_redo(&self) -> bool {
        // let history = &self.history;
        // history.can_redo()
        false
    }
    pub fn redo(&mut self) {
        // let history = &mut self.history;
        // if let Some(Err(e)) = history.redo(&mut self.project_str) {
        //     eprintln!("{}", e)
        // };

        self.compile(self.project_str.clone().as_str());
        self.ui_to_code();
    }

    pub fn open_file(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
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
        // let json = serde_json::to_string_pretty(&self.source);
        // let json_str = json.unwrap_or_else(|e| {
        //     println!("{}", e);
        //     "failed to print".to_string()
        // });
        // self.project_str = json_str;
    }
    pub fn code_to_ui(&mut self) -> Result<(), serde_json::Error> {
        // serde_json::from_str::<Expr>(&self.project_str).map(|expr| {
        //     self.source = Some(expr);
        // })
        Ok(())
    }
    pub fn get_track_for_id_mut(&mut self, id: usize) -> Option<&mut Track> {
        self.project.tracks.get_mut(id)
    }
    pub fn get_track_for_id(&self, id: usize) -> Option<&Track> {
        self.project.tracks.get(id)
    }
    pub fn consume_actions(&mut self) -> bool {
        // self.action_rx
        //     .try_iter()
        //     .map(|action_received| {
        //         self.history
        //             .apply(&mut self.project_str, action_received)
        //             .is_ok()
        //     })
        //     .any(|v| v)
        false
    }
    fn get_default_context(&self) -> ExecContext {
        let plugin = mimium_fns::OtopoiesisPlugin::new(self.project_tx.clone());
        let mut ctx = ExecContext::new([].into_iter(), None, Config::default());
        ctx.add_system_plugin(plugin);
        ctx
    }
    pub fn compile(&mut self, source: &str) -> bool {
        log::debug!("compiling source...{}", source);
        // compile mir for display
        {
            let mut ctx = self.get_default_context();
            ctx.prepare_compiler();
            let mir = ctx.get_compiler_mut().unwrap().emit_mir(source);
            if let Ok(mir) = mir {
                self.project_mir_str = mir.to_string();
            };
        }
        //compile bytecode for display

        let mut ctx = self.get_default_context();
        ctx.prepare_compiler();
        let bytecode = ctx.get_compiler_mut().unwrap().emit_bytecode(source);
        if let Ok(bytecode) = bytecode {
            self.bytecode_str = bytecode.to_string();
        };

        let mut ctx = self.get_default_context();
        let res = ctx.prepare_machine(source);
        let project = self.project_rx.try_iter().last();
        match (res, project) {
            (Ok(()), Some(project)) => {
                self.mimium_ctx = Some(ctx);
                project.slider_map.iter().for_each(|slider| {
                    log::debug!("slider ptr(app) = {:#?}", Arc::as_ptr(slider));
                });

                self.project = project;

                true
            }
            (Err(errs), _) => {
                mimium_lang::utils::error::report(&self.project_str, "".to_symbol(), &errs);
                errs.iter().for_each(|e| {
                    log::error!("{}", e.get_message());
                });
                false
            }
            (Ok(_), None) => {
                log::error!("project is not built");
                self.mimium_ctx = Some(ctx);
                self.project = Project::new(String::new(), 44100);
                true
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PlayOp {
    Play,
    Pause,
    Toggle,
    Halt,
    JumpTo(u64),
}

// #[serde_as]
// #[derive(Serialize, Deserialize, Debug)]
// pub struct Transport {
//     is_playing: atomic::U8,
//     pub time: Arc<atomic::U64>, //in sample
//     playing_history: atomic::U8,
// }

// impl Transport {
//     pub fn new() -> Self {
//         Self::default()
//     }
//     pub fn request_play(&self, p: PlayOp) {
//         self.playing_history.store(self.is_playing.load());
//         self.is_playing.store(p as u8);
//     }
//     pub fn is_playing(&self) -> bool {
//         match PlayOp::from(self.is_playing.load()) {
//             PlayOp::Play => true,
//             PlayOp::Pause | PlayOp::Halt => false,
//         }
//     }
//     pub fn ready_to_trigger(&self) -> Option<PlayOp> {
//         if self.is_playing.load() != self.playing_history.load() {
//             let res = Some(PlayOp::from(self.is_playing.load()));
//             self.playing_history.store(self.is_playing.load());
//             res
//         } else {
//             None
//         }
//     }
// }

// impl Default for Transport {
//     fn default() -> Self {
//         Self {
//             is_playing: atomic::U8::from(2),
//             time: Arc::new(atomic::U64::from(0)),
//             playing_history: atomic::U8::from(2),
//         }
//     }
// }

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct GlobalSetting;

/// A main project data.
#[derive(Debug, Clone)]
pub struct Project {
    pub label: String,
    pub sample_rate: atomic::U64,
    pub current_time: Arc<atomic::U64>, //in sample
    pub tracks: Vec<Track>,
    pub parameters: Vec<SliderId>,
    pub slider_map: Arc<SliderMap>,
}
impl Project {
    pub fn new(label: String, sample_rate: u64) -> Self {
        Self {
            label,
            sample_rate: atomic::U64::from(sample_rate),
            current_time: Arc::new(atomic::U64::from(0)),
            tracks: vec![],
            parameters: vec![],
            slider_map: Default::default(),
        }
    }
}
