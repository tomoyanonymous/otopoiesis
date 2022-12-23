use crate::data;
use crate::gui;

use std::sync::{Arc, Mutex};
pub struct Model {
    pub param: Arc<Mutex<data::AppModel>>,
    timeline: gui::timeline::Model,
    transport: gui::transport::Model,
}

impl Model {
    pub fn new(param: Arc<Mutex<data::AppModel>>) -> Self {
        let p = param.clone();
        let parameter = param.lock().unwrap();
        let proj = parameter.project.clone();
        let sr = proj.sample_rate.load();
        let transport = &parameter.transport;
        Self {
            param: p.clone(),
            timeline: gui::timeline::Model::new(proj, p, transport.time.clone()),
            transport: gui::transport::Model::new(Arc::clone(transport), sr),
        }
    }

    pub fn sync_state(&mut self, track_p: &[data::Track]) {
        self.timeline.sync_state(track_p)
    }
    pub fn show_ui(&mut self, ctx: &egui::Context) {
        let is_mac = ctx.os() == egui::os::OperatingSystem::Mac;

        egui::panel::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("otopoiesis");
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        #[cfg(debug_assertions)]
                        if ui.button("Force Sync Ui State").clicked() {
                            if let Ok(app) = self.param.try_lock() {
                                self.timeline.sync_state(&app.project.tracks);
                            }
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if let Ok(mut app) = self.param.try_lock() {
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
                                    app.history.display(),
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
                                        app.history.display(),
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
                ui.add(&mut self.timeline);
                egui::panel::TopBottomPanel::bottom("footer")
                    .show(ctx, |ui| ui.add(&mut self.transport));
            });
        });
    }
}
