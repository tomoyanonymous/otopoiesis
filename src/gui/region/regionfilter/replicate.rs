use crate::data;
pub struct RegionContent<'a> {
    param: &'a mut data::Region,
    state: &'a mut super::region::State,
}

impl<'a> egui::Widget for RegionContent<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(super::region::Model::new(self.param, self.state))
    }
}
pub struct State {
    pub regions: Vec<super::region::State>,
}
impl State {
    pub fn new(origin: &data::Region, count: u64) -> Self {
        let regions = (0..count)
            .into_iter()
            .map(|i| super::region::State::new(origin, origin.label.clone(), i == 0))
            .collect::<Vec<super::region::State>>();
        Self { regions }
    }
}

pub struct Replicate<'a> {
    pub param: &'a data::ReplicateParam,
    pub origin: &'a mut data::Region,
    state: &'a mut State,
}

impl<'a> Replicate<'a> {
    pub fn new(
        param: &'a data::ReplicateParam,
        origin: &'a mut data::Region,
        state: &'a mut State,
    ) -> Self {
        Self {
            param,
            origin,
            state,
        }
    }
}

impl<'a> egui::Widget for Replicate<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            for region in self.state.regions.iter_mut() {
                ui.add(RegionContent {
                    param: self.origin,
                    state: region,
                });
            }
        })
        .response
    }
}
