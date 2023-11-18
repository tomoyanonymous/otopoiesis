use atomic::SimpleAtomic;
use log::Log;
use std::sync::{Arc, Mutex, OnceLock};

use crate::audio::renderer::{Renderer, RendererBase};
use crate::data::Project;
use crate::script::Expr;
use crate::{audio, data, gui, utils::atomic};

pub(crate) mod filemanager;

extern crate eframe;
extern crate serde_json;
enum EditorMode {
    Code,
    Result,
}
struct Logger {
    pub enabled: atomic::Bool,
    pub data: Arc<Mutex<String>>,
}
impl Logger {
    pub fn new() -> Self {
        Self {
            enabled: atomic::Bool::new(true),
            data: Arc::new(Mutex::new(String::new())),
        }
    }
}
impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.enabled.load() && metadata.level() <= log::STATIC_MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if let Ok(mut txt) = self.data.try_lock() {
            let t = format!(
                "{}:{} -- {}\n",
                record.level(),
                record.target(),
                record.args()
            );
            txt.push_str(&t);

            print!("{}",t);
        }
    }

    fn flush(&self) {
        if let Ok(mut txt) = self.data.try_lock() {
            txt.clear();
        }
    }
}
static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

pub struct Model {
    app: Arc<Mutex<data::AppModel>>,
    audio: Renderer<audio::timeline::Model>,
    compile_err: Option<serde_json::Error>,
    ui: gui::app::State,
    editor_open: bool,
    editor_mode: EditorMode,
    logger_open: bool,
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
        let arg = arg.unwrap_or_default();
        let mut appmodel = data::AppModel::new(data::Transport::new(), data::GlobalSetting {}, arg);
        let _ = appmodel.code_to_ui();
        let ui = gui::app::State::new(&appmodel);
        #[allow(clippy::arc_with_non_send_sync)]
        let app = Arc::new(Mutex::new(appmodel));

        let mut renderer = new_renderer(&app.try_lock().unwrap());

        let _logger = GLOBAL_LOGGER.get_or_init(|| Logger::new());
        if cfg!(debug_assertions)
        {
            log::set_max_level(log::LevelFilter::Debug);
        }else {
            log::set_max_level(log::LevelFilter::Warn);
        }

        log::set_logger(GLOBAL_LOGGER.get().unwrap()).expect("failed to set logger");
        renderer.prepare_play();
        renderer.pause();
        log::debug!("app launched");
        Self {
            audio: renderer,
            app: Arc::clone(&app),
            compile_err: None,
            ui,
            editor_open: false,
            editor_mode: EditorMode::Code,
            logger_open: false,
        }
    }

    pub fn play(&mut self) {
        log::debug!("play");
        self.refresh_audio();

        self.audio.prepare_play();
        self.audio.play();
    }
    pub fn pause(&mut self) {
        log::debug!("pause");
        self.audio.pause();
    }
    fn refresh_audio(&mut self) {
        log::debug!("refresh audio");
        self.audio = new_renderer(&self.app.try_lock().unwrap());
        self.audio.prepare_play();
        self.audio.pause();
    }
    fn sync_transport(&mut self) {
        let t = self.app.try_lock().unwrap().transport.clone();
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
            let mut app = self.app.try_lock().unwrap();
            let need_update = app.consume_actions();
            if need_update {
                let newsrc = app.source.as_ref().unwrap().clone();
                app.compile(newsrc);
                app.ui_to_code();
                self.ui.sync_state(&app.project.tracks);
            }

            ctx.input_mut(|i| {
                if i.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::Z,
                )) && app.can_undo()
                {
                    app.undo();
                    self.ui.sync_state(&app.project.tracks);
                }
                if i.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                    egui::Key::Z,
                )) && app.can_redo()
                {
                    app.redo();
                    self.ui.sync_state(&app.project.tracks);
                }
                if i.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::NONE,
                    egui::Key::Space,
                )) {
                    let op = if app.transport.is_playing() {
                        data::PlayOp::Pause
                    } else {
                        data::PlayOp::Play
                    };
                    app.transport.request_play(op);
                }
                if i.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::NONE,
                    egui::Key::ArrowLeft,
                )) {
                    app.transport.time.store(0);
                    self.audio.prepare_play();
                }
            });
        }

        let style = egui::Style {
            animation_time: 0.2,
            ..Default::default()
        };
        ctx.set_style(style);

        if self.audio.is_playing() {
            //needs constant update while playing
            ctx.request_repaint();
        }

        let _panel = egui::panel::SidePanel::right("Code Viewer")
            .default_width(400.)
            .max_width(1920.)
            .resizable(true)
            .show_animated(ctx, self.editor_open, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let should_refresh_audio = if let Ok(mut app) = self.app.try_lock() {
                        let _ = ui.label("Code Editor");
                        let mut txt = String::new();
                        let widget = match self.editor_mode {
                            EditorMode::Code => {
                                txt = app.source.as_ref().map_or("".to_string(), |src| {
                                    serde_json::to_string_pretty::<Expr>(src).unwrap()
                                });
                                app.project_str = txt.clone();
                                egui::TextEdit::multiline(&mut txt).code_editor()
                            }
                            EditorMode::Result => {
                                txt =
                                    serde_json::to_string_pretty::<Project>(&app.project).unwrap();
                                egui::TextEdit::multiline(&mut txt).code_editor()
                            }
                        };
                        let editor = ui.add_sized(ui.available_size(), widget);
                        if editor.gained_focus() {
                            app.ui_to_code();
                        }
                        let should_refresh_audio = if editor.changed() && editor.lost_focus() {
                            match app.code_to_ui() {
                                Ok(()) => {
                                    self.ui.sync_state(&app.project.tracks);
                                    true
                                }
                                Err(err) => {
                                    self.compile_err = Some(err);
                                    false
                                }
                            }
                        } else {
                            false
                        };
                        if let Some(err) = &self.compile_err {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("failed to evaluate json:{}", err),
                            );
                        }
                        ui.horizontal(|ui| {
                            if ui.button("Code⇆Result").clicked() {
                                self.editor_mode = match self.editor_mode {
                                    EditorMode::Code => EditorMode::Result,
                                    EditorMode::Result => EditorMode::Code,
                                }
                            }
                            if ui.button("Open").clicked() {
                                app.open_file();
                            }
                            ui.add_enabled_ui(app.project_file.is_some(), |ui| {
                                if ui.button("Save").clicked() {
                                    app.save_file();
                                }
                            });
                            if ui.button("Save as").clicked() {
                                app.save_as_file();
                            }
                        });
                        should_refresh_audio
                    } else {
                        false
                    };
                    if should_refresh_audio {
                        self.refresh_audio();
                    }
                });
            });
        egui::panel::SidePanel::right("toggle")
            .min_width(0.)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if let Ok(mut app) = self.app.try_lock() {
                        let text = if self.editor_open { "📕" } else { "📖" };
                        let button = ui.button(text);
                        if button.clicked() {
                            self.editor_open = !self.editor_open;
                            if self.editor_open {
                                app.ui_to_code();
                            }
                        }
                    }
                });
            });
        egui::panel::TopBottomPanel::bottom("Logger")
            .default_height(150.)
            .resizable(true)
            .show_animated(ctx, self.logger_open, |ui| {
                
                ui.vertical(|ui| {

                    ui.label("Log");
                    egui::ScrollArea::vertical().max_height(300.).show(ui,|ui|{

                        if let Ok(mut txt) = GLOBAL_LOGGER.get().unwrap().data.try_lock() {
                            ui.add(
                                egui::TextEdit::multiline(&mut txt as &mut String)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .interactive(false)
                                .desired_rows(5),
                            );
                        }
                    });
                    if ui.button("clear").clicked() {
                        GLOBAL_LOGGER.get().unwrap().flush();
                    }
                });
            });
        egui::panel::TopBottomPanel::bottom("logger_toggle")
            .default_height(30.)
            .resizable(false)
            .show(ctx, |ui| {  

                    ui.toggle_value(&mut self.logger_open, "Console Log");
            });
        //launch main ui
        let mut mainui = gui::app::Model::new(self.app.clone(), &mut self.ui);
        mainui.show_ui(ctx);
        self.sync_transport();
    }
}
