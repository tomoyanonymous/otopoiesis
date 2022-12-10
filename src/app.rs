use std::sync::{Arc, Mutex};

use crate::audio::{
    renderer::{Renderer, RendererBase},
    Component,
};
use crate::{audio, data, gui, utils::atomic};
extern crate eframe;
extern crate serde_json;
pub struct Model {
    app: Arc<Mutex<data::AppModel>>,
    project_str: String,
    code_compiled: serde_json::Result<Arc<data::Project>>,
    audio: Renderer<audio::timeline::Model>,
    ui: gui::app::Model,
    editor_open: bool,
}

impl Model {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let sample_rate = 44100 as u64;

        let project = Arc::new(data::Project {
            sample_rate: atomic::U64::from(sample_rate),
            tracks: Arc::new(Mutex::new(vec![])),
        });
        let transport = Arc::new(data::Transport::new());
        let app = Arc::new(Mutex::new(data::AppModel::new(
            Arc::clone(&transport),
            Arc::new(data::GlobalSetting {}),
            Arc::clone(&project),
        )));
        let ui = gui::app::Model::new(Arc::clone(&app));

        let json = serde_json::to_string_pretty(&project);
        let json_str = json.unwrap_or("failed to parse".to_string());
        let timeline = audio::timeline::Model::new(Arc::clone(&project), Arc::clone(&transport));
        let mut renderer = audio::renderer::create_renderer(
            timeline,
            Some(44100),
            Some(audio::DEFAULT_BUFFER_LEN),
            Arc::clone(&transport),
        );
        renderer.prepare_play();
        renderer.pause();

        Self {
            audio: renderer,
            app: Arc::clone(&app),
            project_str: json_str,
            code_compiled: Ok(project),
            ui,
            editor_open: false,
        }
    }

    pub fn play(&mut self) {
        if !self.audio.is_playing() {
            self.audio.prepare_play();
            {
                let app = &mut self.app.lock().unwrap();
                app.transport.is_playing.store(true);
            }
            self.audio.play();
        }
    }
    pub fn pause(&mut self) {
        if self.audio.is_playing() {
            self.audio.pause();
            {
                let app = &mut self.app.lock().unwrap();
                app.transport.is_playing.store(false);
            }
        }
    }
    fn ui_to_code(&mut self) {
        let app = self.app.lock().unwrap();
        let json = serde_json::to_string_pretty(&app.project);
        let json_str = json.unwrap_or("failed to parse".to_string());
        self.project_str = json_str;
    }
    fn code_to_ui(&mut self) {
        let mut app = self.app.lock().unwrap();
        let proj = serde_json::from_str::<Arc<data::Project>>(&self.project_str);
        self.code_compiled = proj;
        if let Ok(proj) = &self.code_compiled {
            app.project = Arc::clone(proj);
        }
    }
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            let mut app = self.app.lock().unwrap();

            if ctx
                .input_mut()
                .consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::Z,
                ))
            {
                if app.can_undo() {
                    app.undo();
                    self.ui.sync_state();
                }
            }
            if ctx
                .input_mut()
                .consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                    egui::Key::Z,
                ))
            {
                if app.can_redo() {
                    app.redo();
                    self.ui.sync_state();
                }
            }
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

        let mut style = egui::Style::default();
        style.animation_time = 0.2;
        ctx.set_style(style);

        self.ui.show_ui(&ctx);

        if self.audio.is_playing() {
            //needs constant update while playing
            ctx.request_repaint();
        }

        let _panel = egui::panel::SidePanel::right("JSON viewer")
            .default_width(300.)
            .max_width(1920.)
            .resizable(true)
            .show_animated(&ctx, self.editor_open, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let editor = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut self.project_str).code_editor(),
                    );

                    if editor.gained_focus() {
                        self.ui_to_code();
                    }
                    if editor.lost_focus() {
                        self.code_to_ui();
                    }

                    if let Err(err) = &self.code_compiled {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("failed to evaluate json:{}", err.to_string()),
                        );
                    }
                });
            });
        egui::panel::SidePanel::right("toggle")
            .min_width(30.)
            .resizable(false)
            .show(&ctx, |ui| {
                let text = if self.editor_open { "[>]" } else { "[<]" };
                ui.vertical(|ui| {
                    let button = ui.button(text);
                    if button.clicked() {
                        self.editor_open = !self.editor_open;
                        if self.editor_open {
                            self.ui_to_code();
                        }
                    }
                });
            });
    }
}
