use crate::parameter::{FloatParameter, Parameter};

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
        ui.add(
            egui::Slider::from_get_set(
                *range.start() as f64..=*range.end() as f64,
                |v: Option<f64>| {
                    if let Some(n) = v {
                        param.set(n as f32);
                    }
                    param.get() as f64
                },
            )
            .logarithmic(is_log)
        )
    })
    .inner
}
