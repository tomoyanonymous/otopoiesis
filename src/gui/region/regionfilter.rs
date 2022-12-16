pub(crate) mod fadeinout;
pub(crate) mod replicate;
use crate::data;
use crate::gui::region;
use fadeinout::FadeHandle;
use replicate::Replicate;
use std::sync::Arc;

pub enum RegionFilter {
    FadeInOut(FadeHandle),
    Replicate(Replicate),
}

impl RegionFilter {
    pub fn new(filter: Arc<data::RegionFilter>, origin: Arc<data::Region>) -> Self {
        let range = origin.range.clone();

        match filter.as_ref() {
            data::RegionFilter::Gain => todo!(),
            data::RegionFilter::Reverse => todo!(),
            data::RegionFilter::FadeInOut(p) => {
                Self::FadeInOut(FadeHandle::new(p.clone(), Arc::clone(&origin), &range))
            }
            data::RegionFilter::Replicate(_) => todo!(),
        }
    }
}

impl egui::Widget for &mut RegionFilter {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            RegionFilter::FadeInOut(p) => p.ui(ui),
            RegionFilter::Replicate(p) => p.ui(ui),
        }
    }
}
