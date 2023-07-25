use crate::{data, gui};
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
            .map(|i| {
                let is_editable = i == 0;
                super::region::State::new(origin, origin.label.clone(), is_editable)
            })
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
        let scale = move |sec| sec * gui::PIXELS_PER_SEC_DEFAULT;

        ui.horizontal(|ui| {
            for region in self.state.regions.iter_mut() {
                ui.add_sized(
                    egui::vec2(
                        scale(self.origin.range.getrange() as f32),
                        crate::gui::TRACK_HEIGHT,
                    ),
                    RegionContent {
                        param: self.origin,
                        state: region,
                    },
                );
            }
        })
        .response
    }
}
