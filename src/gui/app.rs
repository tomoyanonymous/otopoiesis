use crate::data;
use crate::gui;

use std::sync::{Arc, Mutex, MutexGuard};
pub struct Model {
    pub param: Arc<Mutex<data::AppModel>>,
    timeline: gui::timeline::Model,
}

impl Model {
    pub fn new(param: Arc<Mutex<data::AppModel>>) -> Self {
        let p = param.clone();
        let t = param.lock().unwrap().project.clone();

        Self {
            param: p.clone(),
            timeline: gui::timeline::Model::new(t, p.clone()),
        }
    }

    fn get_model_mut(&self) -> MutexGuard<data::AppModel> {
        self.param.lock().unwrap()
    }
    fn get_timeline(&self) -> Arc<data::Project> {
        self.get_model_mut().project.clone()
    }
    fn get_transport(&self) -> Arc<data::Transport> {
        self.get_model_mut().transport.clone()
    }
    pub fn sync_state(&mut self) {
        self.timeline.sync_state()
    }
    pub fn show_ui(&mut self, ctx: &egui::Context) {
        let is_mac = ctx.os() == egui::os::OperatingSystem::Mac;
        
        egui::panel::TopBottomPanel::top("header").show(&ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("otopoiesis");
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {});
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
        egui::CentralPanel::default().show(&ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(&mut self.timeline);
                egui::panel::TopBottomPanel::bottom("footer").show(&ctx, |ui| {
                    ui.add(gui::transport::Model::new(
                        self.get_transport(),
                        self.get_timeline().sample_rate.load(),
                    ))
                });
            });
        });
    }
}
