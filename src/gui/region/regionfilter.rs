pub(crate) mod fadeinout;
pub(crate) mod replicate;
use std::sync::Arc;

use crate::data;
use crate::gui::region;
use fadeinout::FadeHandle;
use replicate::Replicate;

pub enum RegionFilterState {
    FadeInOut(fadeinout::State),
    Replicate(Replicate),
}

impl RegionFilterState {
    pub fn new(filter: data::RegionFilter, origin: data::Region) -> Self {
        let range = origin.range.clone();

        match filter {
            data::RegionFilter::Gain => todo!(),
            data::RegionFilter::Reverse => todo!(),
            data::RegionFilter::FadeInOut(_) => {
                Self::FadeInOut(fadeinout::State::new(&origin, &range))
            }
            data::RegionFilter::Replicate(r) => Self::Replicate(Replicate::new(r, origin)),
        }
    }
}

pub struct RegionFilter<'a> {
    param: &'a Arc<data::FadeParam>,
    state: &'a mut RegionFilterState,
    origin_ui: &'a mut data::Region,
}
impl<'a> RegionFilter<'a> {
    pub fn new(
        param: &'a Arc<data::FadeParam>,
        state: &'a mut RegionFilterState,
        origin_ui: &'a mut data::Region,
    ) -> Self {
        Self {
            param,
            state,
            origin_ui,
        }
    }
}

impl<'a> egui::Widget for RegionFilter<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self.state {
            RegionFilterState::FadeInOut(p) => {
                ui.add(FadeHandle::new(self.param, self.origin_ui, p))
            }
            RegionFilterState::Replicate(p) => p.ui(ui),
        }
    }
}
