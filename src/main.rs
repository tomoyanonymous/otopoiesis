use crate::utils::AtomicRange;

use eframe;
use egui;
use otopoiesis::*;
use parameter::{FloatParameter, Parameter};
use serde_json;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::audio::{
    renderer::{Renderer, RendererBase},
    Component,
};
use crate::data;
use crate::gui;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::vec2(1200., 900.));
    eframe::run_native(
        "otopoiesis",
        native_options,
        Box::new(|cc| Box::new(Model::new(cc))),
    );
}
struct Model {
    app: Arc<Mutex<data::AppModel>>,
    project_str: String,
    code_compiled: serde_json::Result<Arc<data::Project>>,
    audio: Renderer<audio::timeline::Model>,
}

impl Model {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let region_len = 60000;
        let sample_rate = 44100 as u64;
        let osc_param = Arc::new(data::OscillatorParam {
            amp: FloatParameter::new(1.0, 0.0..=1.0, "amp"),
            freq: FloatParameter::new(440.0, 20.0..=20000.0, "freq"),
            phase: FloatParameter::new(0.0, 0.0..=6.3, "phase"),
        });
        let region_param = Arc::new(data::Region {
            range: AtomicRange::new(1000, 50000),
            max_size: AtomicU64::from(region_len),
            generator: Arc::new(data::Generator::Oscillator(Arc::clone(&osc_param))),
            filters: vec![],
            label: String::from("region0"),
        });
        let project = Arc::new(data::Project {
            sample_rate: AtomicU64::from(sample_rate),
            tracks: Arc::new(Mutex::new(vec![data::Track(Arc::new(Mutex::new(vec![
                Arc::clone(&region_param),
            ])))])),
        });
        let transport = Arc::new(data::Transport::new());
        let app = Arc::new(Mutex::new(data::AppModel::new(
            Arc::clone(&transport),
            Arc::new(data::GlobalSetting {}),
            Arc::clone(&project),
        )));
        let json = serde_json::to_string_pretty(&project);
        let json_str = json.unwrap_or("failed to parse".to_string());
        let mut timeline =
            audio::timeline::Model::new(Arc::clone(&project), Arc::clone(&transport));
        // let sinewave = audio::oscillator::SineModel::new(Arc::clone(&osc_param));
        // let mut region =
        //     audio::region::Model::new(Arc::clone(&region_param), 2, Box::new(sinewave));
        let info = audio::PlaybackInfo {
            sample_rate: sample_rate as u32,
            current_time: 0,
            channels: 2,
            frame_per_buffer: 512,
        };
        timeline.prepare_play(&info);

        let mut renderer = audio::renderer::create_renderer(
            timeline,
            Some(44100),
            Some(512),
            Arc::clone(&transport),
        );
        renderer.prepare_play();
        renderer.pause();

        Self {
            audio: renderer,
            app: Arc::clone(&app),
            project_str: json_str,
            code_compiled: Ok(project),
        }
    }
    pub fn undo(&mut self) {
        let history = &mut self.app.lock().unwrap().history;
        let _ = history.undo(&mut ()).unwrap();
    }
    pub fn redo(&mut self) {
        let history = &mut self.app.lock().unwrap().history;
        let _ = history.redo(&mut ()).unwrap();
    }
    pub fn play(&mut self) {
        if !self.audio.is_playing() {
            let app = &mut self.app.lock().unwrap();
            app.transport.is_playing.store(true, Ordering::Relaxed);
            self.audio.play();
        }
    }
    pub fn pause(&mut self) {
        if self.audio.is_playing() {
            self.audio.pause();
            let app = &mut self.app.lock().unwrap();
            app.transport.is_playing.store(false, Ordering::Relaxed);
        }
    }
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if ctx
            .input_mut()
            .consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Z,
            ))
        {
            self.undo();
        }
        if ctx
            .input_mut()
            .consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                egui::Key::Z,
            ))
        {
            self.redo();
        }
        if ctx
            .input_mut()
            .consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Space,
            ))
        {
            if self.audio.is_playing() {
                self.pause();
            } else {
                self.audio.prepare_play();
                self.play();
            }
        }
        if ctx
            .input_mut()
            .consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::ArrowLeft,
            ))
        {
            self.audio.rewind();
            self.audio.prepare_play();
        }
        let mut app_gui = gui::app::Model {
            param: Arc::clone(&self.app),
        };
        app_gui.show_ui(&ctx);
        let _panel = egui::panel::SidePanel::right("JSON viewer")
            .default_width(300.)
            .min_width(300.)
            .max_width(1920.)
            .resizable(true)
            .show(&ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let editor = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut self.project_str).code_editor(),
                    );
                    let mut app = self.app.lock().unwrap();
                    if editor.gained_focus() {
                        let json = serde_json::to_string_pretty(&app.project);
                        let json_str = json.unwrap_or("failed to parse".to_string());
                        self.project_str = json_str;
                    }
                    if editor.lost_focus() {
                        let proj = serde_json::from_str::<Arc<data::Project>>(&self.project_str);
                        self.code_compiled = proj;
                        if let Ok(proj) = &self.code_compiled {
                            app.project = Arc::clone(proj);
                        }
                    }

                    if let Err(err) = &self.code_compiled {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("failed to evaluate json:{}", err.to_string()),
                        );
                    }

                    // ui.code_editor(model.
                });
            });
    }
}
