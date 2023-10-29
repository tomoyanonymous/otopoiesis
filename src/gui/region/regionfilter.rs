pub(crate) mod fadeinout;
pub(crate) mod replicate;

use crate::gui::region;

// pub enum RegionFilterState {
//     FadeInOut(fadeinout::State),
//     Replicate(Replicate),
// }

// impl RegionFilterState {
//     pub fn new(filter: data::RegionFilter, origin: data::Region) -> Self {
//         let range = origin.range.clone();

//         match filter {
//             data::RegionFilter::Gain => todo!(),
//             data::RegionFilter::Reverse => todo!(),
//             data::RegionFilter::FadeInOut(_) => {
//                 Self::FadeInOut(fadeinout::State::new(&origin, &range))
//             }
//             data::RegionFilter::Replicate(r) => Self::Replicate(Replicate::new(r, origin)),
//         }
//     }
// }

// pub struct RegionFilter<'a> {
//     param: &'a Arc<data::FadeParam>,
//     state: &'a mut RegionFilterState,
//     origin_ui: &'a mut data::Region,
// }

pub enum RegionFilter<'a> {
    FadeInOut(fadeinout::FadeInOut<'a>),
    Replicate(replicate::Replicate<'a>),
}
pub enum RegionFilterState {
    FadeInOut(fadeinout::State),
    Replicate(replicate::State),
}

// impl<'a> RegionFilter<'a> {
//     pub fn new(
//         param: &'a Arc<data::FadeParam>,
//         state: &'a mut RegionFilterState,
//         origin_ui: &'a mut data::Region,
//     ) -> Self {
//         Self {
//             param,
//             state,
//             origin_ui,
//         }
//     }
// }

impl<'a> egui::Widget for RegionFilter<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            RegionFilter::FadeInOut(p) => ui.add(p),
            RegionFilter::Replicate(p) => ui.add(p),
        }
    }
}
