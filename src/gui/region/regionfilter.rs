pub(crate) mod fadeinout;
pub(crate) mod replicate;
use crate::data;
use crate::gui::region;
use fadeinout::FadeHandle;
use replicate::Replicate;

pub enum RegionFilter {
    FadeInOut(FadeHandle),
    Replicate(Replicate),
}

impl RegionFilter {
    pub fn new(filter: data::RegionFilter, origin: data::Region) -> Self {
        let range = origin.range.clone();

        match filter {
            data::RegionFilter::Gain => todo!(),
            data::RegionFilter::Reverse => todo!(),
            data::RegionFilter::FadeInOut(p) => Self::FadeInOut(FadeHandle::new(p, origin, &range)),
            data::RegionFilter::Replicate(r) => Self::Replicate(Replicate::new(r, origin)),
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
