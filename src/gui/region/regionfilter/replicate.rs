use crate::data;

use std::sync::Arc;
pub struct Replicate {
    pub param: Arc<data::ReplicateParam>,
    pub origin: Arc<data::Region>,
    regions: Vec<super::region::Model>,
}

impl Replicate {
    pub fn new(param: Arc<data::ReplicateParam>, origin: Arc<data::Region>) -> Self {
        let regions = (0..param.count.load())
            .into_iter()
            .map(|i| {
                super::region::Model::new(origin.clone(), format!("{}_rep_{}", origin.label, i))
            })
            .collect::<Vec<_>>();
        Self {
            param,
            origin,
            regions,
        }
    }
}

impl egui::Widget for &mut Replicate {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            for region in self.regions.iter_mut() {
                ui.add(region);
            }
        })
        .response
    }
}
