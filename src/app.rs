use atomic::SimpleAtomic;
use std::sync::{Arc, Mutex};

use crate::audio::renderer::{Renderer, RendererBase};
use crate::{audio, data, gui, utils::atomic};

pub(crate) mod filemanager;

extern crate eframe;
extern crate serde_json;
pub struct Model {
    app: Arc<Mutex<data::AppModel>>,
    project_str: String,
    audio: Renderer<audio::timeline::Model>,
    compile_err: Option<serde_json::Error>,
    ui: gui::app::State,
    editor_open: bool,
}

fn new_renderer(app: &data::AppModel) -> Renderer<audio::timeline::Model> {
    let timeline = audio::timeline::Model::new(app.project.clone(), Arc::clone(&app.transport));
    audio::renderer::create_renderer(
        timeline,
        Some(44100),
        Some(audio::DEFAULT_BUFFER_LEN),
        Arc::clone(&app.transport),
    )
}

impl Model {
    pub fn new(_cc: &eframe::CreationContext<'_>, arg: Option<data::LaunchArg>) -> Self {
        let sample_rate = 44100;
        let project = data::Project {
            sample_rate: atomic::U64::from(sample_rate),
            tracks: vec![],
        };

        let json = serde_json::to_string_pretty(&project);
        let json_str = json.unwrap_or_else(|e| {
            println!("{}", e);
            "failed to print".to_string()
        });
        let arg = arg.unwrap_or_default();
        let appmodel =
            data::AppModel::new(data::Transport::new(), data::GlobalSetting {}, arg, project);
        let ui = gui::app::State::new(&appmodel);
        let app = Arc::new(Mutex::new(appmodel));

        let mut renderer = new_renderer(&app.lock().unwrap());

        renderer.prepare_play();
        renderer.pause();

        Self {
            audio: renderer,
            app: Arc::clone(&app),
            project_str: json_str,
            compile_err: None,
            ui,
            editor_open: false,
        }
    }

    pub fn play(&mut self) {
        self.refresh_audio();

        self.audio.prepare_play();
        self.audio.play();
    }
    pub fn pause(&mut self) {
        self.audio.pause();
    }
    fn ui_to_code(&mut self) {
        let app = self.app.lock().unwrap();
        let json = serde_json::to_string_pretty(&app.project);
        let json_str = json.unwrap_or_else(|e| {
            println!("{}", e);
            "failed to print".to_string()
        });
        self.project_str = json_str;
    }
    fn code_to_ui(&mut self) {
        if let Ok(mut app) = self.app.lock() {
            match serde_json::from_str::<data::Project>(&self.project_str) {
                Ok(proj) => {
                    app.project = proj.clone();
                    self.compile_err = None;
                    self.ui.sync_state(&proj.tracks);
                }
                Err(err) => {
                    self.compile_err = Some(err);
                }
            }
        }
        self.refresh_audio();
    }
    fn refresh_audio(&mut self) {
        self.audio = new_renderer(&self.app.lock().unwrap());
        self.audio.prepare_play();
        self.audio.pause();
    }

    fn _respawn_ui(&mut self) {
        self.ui = gui::app::State::new(&self.app.lock().unwrap());
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
                && app.can_undo()
            {
                app.undo();
                self.ui.sync_state(&app.project.tracks);
            }
            if ctx
                .input_mut()
                .consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                    egui::Key::Z,
                ))
                && app.can_redo()
            {
                app.redo();
                self.ui.sync_state(&app.project.tracks);
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
        let mut mainui = gui::app::Model::new(self.app.clone(), &mut self.ui);
        mainui.show_ui(ctx);
        self.sync_transport();

        let style = egui::Style {
            animation_time: 0.2,
            ..Default::default()
        };
        ctx.set_style(style);

        if self.audio.is_playing() {
            //needs constant update while playing
            ctx.request_repaint();
        }

        let _panel = egui::panel::SidePanel::right("JSON viewer")
            .default_width(400.)
            .max_width(1920.)
            .resizable(true)
            .show_animated(ctx, self.editor_open, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let _ = ui.label("Code Editor");
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

                    if let Some(err) = &self.compile_err {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("failed to evaluate json:{}", err),
                        );
                    }
                });
            });
        egui::panel::SidePanel::right("toggle")
            .min_width(0.)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let text = if self.editor_open { "📕" } else { "📖" };
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
