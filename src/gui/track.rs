use crate::data;
use crate::gui::region;
use nannou_egui::egui;
use std::sync::Arc;
pub struct Model {
    pub param: Arc<data::Track>,
}

impl egui::Widget for Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            //regions maybe overlap to each other, so we need to split layer
            for (i, region) in self.param.0.iter().enumerate() {
                ui.add(&mut region::Model::new(Arc::clone(region)))
                    .interact(egui::Sense::click_and_drag());
            }
        })
        .response
    }
}
