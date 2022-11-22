use crate::data;
use crate::gui;
use crate::parameter::Parameter;
use crate::utils::AtomicRange;
use crate::*;
use nannou_egui::egui;
use std::sync::Arc;

pub struct Model {
    time: u64,
    params: Arc<data::Project>
}

impl egui::Widget for Model {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.button("text")
    }
}
