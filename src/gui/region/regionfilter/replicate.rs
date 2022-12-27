use crate::data;
use crate::utils::atomic::SimpleAtomic;
enum RegionContent {
    Editable(data::Region, super::region::State), //todo
    NonEditable(super::region::ReadOnlyModel),
}
impl egui::Widget for &mut RegionContent {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            RegionContent::Editable(p, m) => ui.add(super::region::Model::new(p, m)),
            RegionContent::NonEditable(m) => m.ui(ui),
        }
    }
}

pub struct Replicate {
    pub param: data::ReplicateParam,
    pub origin: data::Region,
    regions: Vec<RegionContent>,
}

impl Replicate {
    pub fn new(param: data::ReplicateParam, origin: data::Region) -> Self {
        let regions = (0..param.count.load())
            .into_iter()
            .map(|i| {
                if i == 0 {
                    RegionContent::Editable(
                        origin.clone(),
                        super::region::State::new(&origin, origin.label.clone()),
                    )
                } else {
                    RegionContent::NonEditable(super::region::ReadOnlyModel::new(&origin))
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
