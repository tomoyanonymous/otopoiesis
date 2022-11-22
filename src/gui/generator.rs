
use crate::data;

use egui::Color32;
use std::sync::Arc;
// use nannou_egui::*;
use nannou_egui::egui;

struct Model{
    params:Arc<data::Generator>
}
fn default_graph(
    label: impl std::hash::Hash,
    iter: impl Iterator<Item = egui::plot::Value>,
) -> egui::plot::Plot {
    let line = egui::plot::Line::new(egui::plot::Values::from_values_iter(iter));
    egui::plot::Plot::new(label)
        .line(line)
        .allow_drag(false)
        .allow_zoom(false)
}

impl egui::Widget for &Model{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // let center = ui.add_sized(
        //     region_size,
        //     default_graph(
        //         "region",
        //         self.samples.iter().enumerate().map(|(i, s)| {
        //             let x = nannou::math::map_range(
        //                 i as f32,
        //                 0.,
        //                 self.samples.len() as f32,
        //                 0.,
        //                 x_size,
        //             );
        //             let y = *s * 100.0 * self.get_current_amp();
        //             egui::plot::Value::new(x, y)
        //         }),
        //     ),
        // );
        // match self.params.as_ref() {
        //     data::Generator::Oscillator(osc)=>{
                
        //     }
        // }
        ui.label("dummy")
    }
}