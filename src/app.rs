use std::sync::{Arc, Mutex};

use crate::audio::renderer::{Renderer, RendererBase};
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

fn new_renderer(app: &data::AppModel) -> Renderer<audio::timeline::Model> {
    let timeline =
        audio::timeline::Model::new(Arc::clone(&app.project), Arc::clone(&app.transport));
    audio::renderer::create_renderer(
        timeline,
        Some(44100),
        Some(audio::DEFAULT_BUFFER_LEN),
        Arc::clone(&app.transport),
    )
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
        let mut renderer = new_renderer(&app.lock().unwrap());

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
        self.audio.prepare_play();
        self.audio.play();
    }
    pub fn pause(&mut self) {
        self.audio.pause();
    }
    fn ui_to_code(&mut self) {
        let app = self.app.lock().unwrap();
        let json = serde_json::to_string_pretty(&app.project);
        let json_str = json.unwrap_or("failed to parse".to_string());
        self.project_str = json_str;
    }
    fn code_to_ui(&mut self) {
        if let Ok(mut app) = self.app.lock() {
            let proj = serde_json::from_str::<Arc<data::Project>>(&self.project_str);
            self.code_compiled = proj;
            if let Ok(proj) = &self.code_compiled {
                app.project = Arc::clone(proj);
                self.ui.sync_state(&proj.tracks);
            }
        }
        self.refresh_audio();
    }
    fn refresh_audio(&mut self) {
        self.audio = new_renderer(&self.app.lock().unwrap());
        self.audio.prepare_play();
        self.audio.pause();
    }
    fn sync_transport(&mut self) {
        let t = self.app.lock().unwrap().transport.clone();
        if let Some(b) = t.ready_to_trigger() {
            match b {
                data::PlayOp::Play => self.play(),
                data::PlayOp::Pause => self.pause(),
                data::PlayOp::Halt => {
                    self.pause();
                    self.audio.rewind();
                }
            }
        } else {
            //do nothing
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
                    self.ui.sync_state(&app.project.tracks);
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
                    self.ui.sync_state(&app.project.tracks);
                }
            }
            if ctx
                .input_mut()
                .consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::NONE,
                    egui::Key::Space,
                ))
            {
                let op = if app.transport.is_playing() {
                    data::PlayOp::Pause
                } else {
                    data::PlayOp::Play
                };
                app.transport.request_play(op);
            }
            if ctx
                .input_mut()
                .consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::NONE,
                    egui::Key::ArrowLeft,
                ))
            {
                app.transport.time.store(0);
                self.audio.prepare_play();
            }
        }
        self.sync_transport();

        let mut style = egui::Style::default();
        style.animation_time = 0.2;
        ctx.set_style(style);

        self.ui.show_ui(&ctx);

        if self.audio.is_playing() {
            //needs constant update while playing
            ctx.request_repaint();
        }

        let _panel = egui::panel::SidePanel::right("JSON viewer")
            .default_width(400.)
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
