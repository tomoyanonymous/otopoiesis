use crate::{
    parameter::{FloatParameter, Parameter},
    utils::SimpleAtomic,
};

pub(crate) fn slider_from_parameter(
    param: &FloatParameter,
    is_log: bool,
    ui: &mut egui::Ui,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.group(|ui| {
            ui.set_width(50.);
            ui.centered_and_justified(|ui| {
                ui.label(param.get_label());
            })
        });
        let range = &param.range;
        let start = ui.add(
            egui::DragValue::from_get_set(|v: Option<f64>| {
                if let Some(n) = v {
                    range.start().store(n as f32);
                }
                range.start().load() as f64
            })
            .max_decimals(5),
        );
        let main = ui.add(
            egui::Slider::from_get_set(
                range.start().load() as f64..=range.end().load() as f64,
                |v: Option<f64>| {
                    if let Some(n) = v {
                        param.set(n as f32);
                    }
                    param.get() as f64
                },
            )
            .show_value(false)
            .logarithmic(is_log),
        );
        let end = ui.add(
            egui::DragValue::from_get_set(|v: Option<f64>| {
                if let Some(n) = v {
                    range.end().store(n as f32);
                }
                range.end().load() as f64
            })
            .max_decimals(5),
        );
        start.union(main.union(end))
    })
    .inner
}
