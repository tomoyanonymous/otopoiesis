use crate::audio::component::mimium_component;
use crate::audio::renderer::PlayState;
use crate::data::PlayOp;
use crate::utils::{GLOBAL_LOGGER, Logger};
use crate::{atomic, audio, data, gui};
use atomic::SimpleAtomic;
use audio::renderer::{Renderer, RendererBase};
use log::Log;
use mimium_lang::{Config, ExecContext};
use std::sync::{Arc, Mutex, mpsc};

pub(crate) mod filemanager;

extern crate eframe;
extern crate serde_json;

#[derive(PartialEq, Clone, Copy, Debug)]
enum EditorMode {
    Code,
    Result,
    Mir,
    ByteCode,
}

use mimium_component::MimiumComponent;
pub struct Model {
    app: data::AppModel,
    audio: Renderer<MimiumComponent>,
    compile_err: Option<serde_json::Error>,
    // ui: gui::app::State,
    editor_open: bool,
    editor_mode: EditorMode,
    logger_open: bool,
}

fn new_renderer(app: &mut data::AppModel) -> Renderer<MimiumComponent> {
    if app.mimium_ctx.as_mut().unwrap().get_vm().is_none() {
        let src = app.project_str.clone();
        app.compile(&src);
    }
    let ctx = app.mimium_ctx.as_mut().unwrap();

    let vm = ctx.take_vm().unwrap_or_else(|| {
        log::error!("failed to compile code, use dummy dsp");
        let dummy_src = "let dsp = | | 0.0 ";
        app.compile(&dummy_src);
        app.mimium_ctx.as_mut().unwrap().take_vm().unwrap()
    });
    let component = MimiumComponent::new(vm);
    audio::renderer::create_renderer(
        component,
        Some(app.project.sample_rate.load() as u32),
        Some(audio::DEFAULT_BUFFER_LEN),
        app.project.current_time.clone(),
    )
}

impl Model {
    pub fn new(cc: &eframe::CreationContext<'_>, arg: Option<data::LaunchArg>) -> Self {
        let arg = arg.unwrap_or_default();
        let (sender, receriver) = mpsc::channel();
        Self::setup_custom_fonts(&cc.egui_ctx);
        let mut appmodel =
            data::AppModel::new(sender, PlayState::Stopped, data::GlobalSetting, arg);
        let _ = appmodel.code_to_ui();
        let initsrc = &appmodel.project_str.clone();
        appmodel.compile(&initsrc);
        // let ui = gui::app::State::new(&appmodel);
        // #[allow(clippy::arc_with_non_send_sync)]
        // let mut app = Arc::new(Mutex::new(appmodel));

        let mut renderer = new_renderer(&mut appmodel);

        let _logger = GLOBAL_LOGGER.get_or_init(|| Logger::new());
        if cfg!(debug_assertions) {
            log::set_max_level(log::LevelFilter::Debug);
        } else {
            log::set_max_level(log::LevelFilter::Warn);
        }

        log::set_logger(GLOBAL_LOGGER.get().unwrap()).expect("failed to set logger");
        log::debug!("app launched");
        Self {
            audio: renderer,
            app: appmodel,
            compile_err: None,
            editor_open: false,
            editor_mode: EditorMode::Code,
            logger_open: false,
        }
    }
    fn setup_custom_fonts(ctx: &egui::Context) {
        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            "my_font".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/fonts/NotoSansJP-VariableFont_wght.ttf"
            )))),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("my_font".to_owned());

        // Tell egui to use these fonts:
        ctx.set_fonts(fonts);
    }
    pub fn play(&mut self) {
        log::debug!("play");
        self.refresh_audio();

        self.audio.prepare_play();
        self.audio.control(PlayOp::Play);
    }
    pub fn pause(&mut self) {
        log::debug!("pause");
        self.audio.control(PlayOp::Pause);
    }
    pub fn halt(&mut self) {
        log::debug!("halt");
        self.audio.control(PlayOp::Halt);
    }
    fn refresh_audio(&mut self) {
        log::debug!("refresh audio");
        let (sender, receiver) = mpsc::channel();
        self.app.playop_queue = sender;
        self.audio = new_renderer(&mut self.app);
        self.audio.prepare_play();
        self.audio.control(PlayOp::Pause);
    }
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let need_update = self.app.consume_actions();
        if need_update {
            let newsrc = self.app.project_str.clone();
            self.app.compile(newsrc.as_str());
            self.app.ui_to_code();
            // self.ui.sync_state(&app.project.tracks);
        }
        match self.app.playstate {
            PlayState::Playing if !self.audio.is_playing() => {
                self.play();
            }
            PlayState::Paused if self.audio.is_playing() => {
                self.pause();
            }
            PlayState::Stopped if self.audio.is_playing() => {
                self.halt();
            }
            _ => {}
        };

        ctx.input_mut(|i| {
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Z,
            )) && self.app.can_undo()
            {
                self.app.undo();
                // self.ui.sync_state(&app.project.tracks);
            }
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                egui::Key::Z,
            )) && self.app.can_redo()
            {
                self.app.redo();
                // self.ui.sync_state(&app.project.tracks);
            }
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Space,
            )) && !self.editor_open
            //do not play/pause when editor is focused to prevent from misediting
            {
                if self.audio.is_playing() {
                    self.app.playstate = PlayState::Paused;
                    self.pause();
                } else {
                    self.app.playstate = PlayState::Playing;
                    self.play();
                }
            }
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::ArrowLeft,
            )) {
                self.halt();
            }
        });

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
                    let _ = ui.label("Code Editor");
                    let should_refresh_audio = {
                        ui.horizontal(|ui| {
                            let button = ui.menu_button("Code Menu", |ui| {
                                ui.selectable_value(
                                    &mut self.editor_mode,
                                    EditorMode::Code,
                                    "Code",
                                );
                                ui.selectable_value(
                                    &mut self.editor_mode,
                                    EditorMode::Result,
                                    "Result",
                                );
                                ui.selectable_value(&mut self.editor_mode, EditorMode::Mir, "Mir");
                                ui.selectable_value(
                                    &mut self.editor_mode,
                                    EditorMode::ByteCode,
                                    "ByteCode",
                                );
                            });
                            if ui.button("Open").clicked() {
                                self.app.open_file();
                            }
                            ui.add_enabled_ui(self.app.project_file.is_some(), |ui| {
                                if ui.button("Save").clicked() {
                                    self.app.save_file();
                                }
                            });
                            if ui.button("Save as").clicked() {
                                self.app.save_as_file();
                            }
                        });
                        let widget = match self.editor_mode {
                            EditorMode::Code | EditorMode::Result => {
                                egui::TextEdit::multiline(&mut self.app.project_str)
                                    .font(egui::TextStyle::Monospace) // for cursor height
                                    .code_editor()
                                    .lock_focus(false)
                            }
                            EditorMode::Mir => {
                                egui::TextEdit::multiline(&mut self.app.project_mir_str)
                                    .font(egui::TextStyle::Monospace) // for cursor height
                                    .interactive(false)
                            }
                            EditorMode::ByteCode => {
                                egui::TextEdit::multiline(&mut self.app.bytecode_str)
                                    .font(egui::TextStyle::Monospace) // for cursor height
                                    .interactive(false)
                            }
                        };
                        let editor = ui.add_sized(ui.available_size(), widget);
                        if editor.gained_focus() {
                            self.app.ui_to_code();
                        }
                        let should_refresh_audio = if editor.changed() && editor.lost_focus() {
                            match self.app.code_to_ui() {
                                Ok(()) => {
                                    // self.ui.sync_state(&app.project.tracks);
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

                        should_refresh_audio
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
                    let app = &mut self.app;
                    let text = if self.editor_open { "📕" } else { "📖" };
                    let button = ui.button(text);
                    if button.clicked() {
                        self.editor_open = !self.editor_open;
                        if self.editor_open {
                            app.ui_to_code();
                        }
                    }
                });
            });
        egui::panel::TopBottomPanel::bottom("Logger")
            .default_height(150.)
            .max_height(400.)
            .min_height(100.)
            .resizable(true)
            .show_animated(ctx, self.logger_open, |ui| {
                ui.vertical(|ui| {
                    ui.label("Log");
                    egui::ScrollArea::vertical()
                        .max_height(300.)
                        .show(ui, |ui| {
                            if let Ok(mut data) = GLOBAL_LOGGER.get().unwrap().data.try_lock() {
                                data.iter_mut().rev().for_each(|d| {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut d.0)
                                            .desired_width(f32::MAX)
                                            .interactive(false)
                                            .text_color(Logger::get_color(d.1)),
                                    );
                                });
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

        let mut mainui = gui::app::Model::new(&mut self.app);
        mainui.show_ui(ctx);
    }
}
