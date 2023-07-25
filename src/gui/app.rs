use crate::data;
use crate::gui;

use crate::utils::atomic::SimpleAtomic;
use std::sync::{Arc, Mutex};
pub struct State {
    timeline: gui::timeline::State,
    transport: gui::transport::Model,
}

impl State {
    pub fn new(param: &data::AppModel) -> Self {
        let sr = param.project.sample_rate.load();
        let transport = &param.transport;
        let transport = gui::transport::Model::new(Arc::clone(transport), sr);
        let timeline =
            gui::timeline::State::new(&param.project.tracks, transport.param.time.clone(), sr);
        Self {
            timeline,
            transport,
        }
    }
    pub fn sync_state(&mut self, track_p: &[data::Track]) {
        self.timeline.sync_state(track_p)
    }
}
pub struct Model<'a> {
    pub app: Arc<Mutex<data::AppModel>>,
    state: &'a mut State,
}

impl<'a> Model<'a> {
    pub fn new(app: Arc<Mutex<data::AppModel>>, state: &'a mut State) -> Self {
        Self { app, state }
    }

    pub fn show_ui(&mut self, ctx: &egui::Context) {
        let is_mac = ctx.os() == egui::os::OperatingSystem::Mac;

        egui::panel::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("otopoiesis");
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Force Sync Ui State(Debug)").clicked() {
                            #[cfg(debug_assertions)]
                            self.state
                                .timeline
                                .sync_state(&self.app.lock().unwrap().project.tracks);
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if let Ok(mut app) = self.app.lock() {
                            let undo_sk =
                                egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
                            let redo_sk = egui::KeyboardShortcut::new(
                                egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                                egui::Key::Z,
                            );
                            let str = undo_sk.format(&egui::ModifierNames::NAMES, is_mac);
                            let undobutton = ui.add_enabled(
                                app.can_undo(),
                                egui::Button::new(format!(
                                    "Undo {} | {}",
                                    app.history.undo_text().unwrap_or_default(),
                                    str
                                )),
                            );
                            // list truncated history here
                            if undobutton.clicked() {
                                app.undo();
                            };
                            let str = redo_sk.format(&egui::ModifierNames::NAMES, is_mac);
                            if ui
                                .add_enabled(
                                    app.can_redo(),
                                    egui::Button::new(format!(
                                        "Redo {} | {}",
                                        app.history.redo_text().unwrap_or_default(),
                                        str
                                    )),
                                )
                                .clicked()
                            {
                                app.redo();
                            }
                        }
                    })
                })
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(super::timeline::Model::new(
                    self.app.clone(),
                    &mut self.state.timeline,
                ));
                egui::panel::TopBottomPanel::bottom("footer")
                    .show(ctx, |ui| ui.add(&mut self.state.transport));
            });
        });
    }
}
