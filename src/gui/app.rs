use std::sync::Arc;

use crate::audio::renderer::PlayState;
use crate::data;
use crate::gui;

use crate::atomic::SimpleAtomic;

// pub struct State {
//     timeline: gui::timeline::State,
//     transport: gui::transport::Model,
// }

// impl State {
//     pub fn new(param: &data::AppModel) -> Self {
//         let sr = param.project.sample_rate.load();
//         let transport = &param.transport;
//         let transport = gui::transport::Model::new(Arc::clone(transport), sr);
//         let timeline =
//             gui::timeline::State::new(&param.project.tracks, transport.param.time.clone(), sr);
//         Self {
//             timeline,
//             transport,
//         }
//     }
//     pub fn sync_state(&mut self, track_p: &[data::Track]) {
//         self.timeline.sync_state(track_p)
//     }
// }
pub struct Model<'a> {
    pub app: &'a mut data::AppModel,
}

impl<'a> Model<'a> {
    pub fn new(app: &'a mut data::AppModel) -> Self {
        Self { app }
    }
    fn consume_shortcuts(&mut self, ctx: &egui::Context) {
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
            ))
            //do not play/pause when editor is focused to prevent from misediting
            {
                if self.app.is_playing() {
                    self.app.playstate = PlayState::Paused;
                    self.app.pause();
                } else {
                    self.app.playstate = PlayState::Playing;
                    self.app.code_to_ui();
                    self.app.play();
                }
            }
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::ArrowLeft,
            )) {
                self.app.halt();
            }
        });
    }
    pub fn show_ui(&mut self, ctx: &egui::Context) {
        let is_mac = ctx.os() == egui::os::OperatingSystem::Mac;

        egui::panel::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("otopoiesis");
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            self.app.open_file();
                        }
                        ui.add_enabled_ui(self.app.project_file.is_some(), |ui| {
                            if ui.button("Save").clicked() {
                                self.app.save_as_file();
                            }
                        });
                        if ui.button("Save as").clicked() {
                            self.app.save_as_file();
                        }

                        // if ui.button("Force Sync Ui State(Debug)").clicked() {
                        //     #[cfg(debug_assertions)]
                        //     self.app.try_lock().map(|app| {

                        //         // .sync_state(&self.app.try_lock().unwrap().project.tracks);
                        //     })
                        // }
                    });
                    ui.menu_button("Edit", |ui| {
                        let undo_sk =
                            egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
                        let redo_sk = egui::KeyboardShortcut::new(
                            egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                            egui::Key::Z,
                        );
                        let str = undo_sk.format(&egui::ModifierNames::NAMES, is_mac);
                        let undobutton = ui.add_enabled(
                            self.app.can_undo(),
                            egui::Button::new(format!(
                                "Undo  | {}",
                                // app.history.undo_text().unwrap_or_default(),
                                str
                            )),
                        );
                        // list truncated history here
                        if undobutton.clicked() {
                            self.app.undo();
                        };
                        let str = redo_sk.format(&egui::ModifierNames::NAMES, is_mac);
                        if ui
                            .add_enabled(
                                self.app.can_redo(),
                                egui::Button::new(format!(
                                    "Redo | {}",
                                    // app.history.redo_text().unwrap_or_default(),
                                    str
                                )),
                            )
                            .clicked()
                        {
                            self.app.redo();
                        }
                    })
                })
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.consume_shortcuts(ctx);
            if self.app.is_playing() {
                //needs constant update while playing
                ctx.request_repaint();
            }
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(super::timeline::Model::new(self.app));

                egui::panel::TopBottomPanel::bottom("footer").show(ctx, |ui| {
                    let transportmodel = gui::transport::Model::new(
                        &mut self.app.playstate,
                        self.app.project.sample_rate.load(),
                        self.app.project.current_time.clone(),
                    );
                    ui.add(transportmodel);
                });
            });
        });
    }
}
