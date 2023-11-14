pub(crate) mod fadeinout;
pub(crate) mod replicate;

use crate::gui::region;

pub enum RegionFilter<'a> {
    FadeInOut(fadeinout::FadeInOut<'a>),
    Replicate(replicate::Replicate<'a>),
}
pub enum RegionFilterState {
    FadeInOut(fadeinout::State),
    Replicate(replicate::State),
}

impl<'a> egui::Widget for RegionFilter<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            RegionFilter::FadeInOut(p) => ui.add(p),
            RegionFilter::Replicate(p) => ui.add(p),
        }
    }
}
