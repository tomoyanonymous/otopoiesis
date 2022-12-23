use crate::data;

use std::sync::Arc;
enum RegionContent {
    Editable(super::region::Model),
    NonEditable(super::region::ReadOnlyModel),
}
impl egui::Widget for &mut RegionContent {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            RegionContent::Editable(m) => m.ui(ui),
            RegionContent::NonEditable(m) => m.ui(ui),
        }
    }
}

pub struct Replicate {
    pub param: Arc<data::ReplicateParam>,
    pub origin: Arc<data::Region>,
    regions: Vec<RegionContent>,
}

impl Replicate {
    pub fn new(param: Arc<data::ReplicateParam>, origin: Arc<data::Region>) -> Self {
        let regions = (0..param.count.load())
            .into_iter()
            .map(|i| {
                if i == 0 {
                    RegionContent::Editable(super::region::Model::new(
                        origin.clone(),
                        origin.label.clone(),
                    ))
                } else {
                    RegionContent::NonEditable(super::region::ReadOnlyModel::new(origin.as_ref()))
                }
            })
            .collect::<Vec<RegionContent>>();
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
