use crate::action::{self, Action};
use crate::data::{
    self,
    script::{Expr, Type, Value},
};

use crate::parameter::{FloatParameter, Parameter, RangedNumeric};
use std::sync::{mpsc, Arc};

fn make_region(trackid: usize, pos: f64, c: String) -> Value {
    let region = Value::Region(
        pos,
        pos + 1.0,
        Value::ExtFunction(c).into(),
        format!("region{}", trackid + 1),
        Type::Unknown,
    );

    Value::Function(
        vec![],
        Expr::App(
            Expr::Literal(Value::ExtFunction("fadeinout".to_string())).into(),
            vec![
                Expr::Literal(region),
                Expr::Literal(Value::Parameter(Arc::new(
                    FloatParameter::new(0.1, "time_in").set_range(0.0..=1000.),
                ))),
                Expr::Literal(Value::Parameter(Arc::new(
                    FloatParameter::new(0.1, "time_out").set_range(0.0..=1000.),
                ))),
            ],
        )
        .into(),
    )
}

pub fn add_region_button(
    trackid: usize,
    pos: f64,
    sender: &mpsc::Sender<Action>,
    ui: &mut egui::Ui,
) -> egui::Response {
    ui.menu_button("+", |ui| {
        let id = ui.auto_id_with("osckind");
        let mut osckind = ui
            .ctx()
            .data_mut(|d| d.get_persisted(id))
            .unwrap_or("sinewave".to_string());
        let (addosc, osckind) = ui
            .horizontal(|ui| {
                let addosc = ui.button("~ Add oscillator");
                let _ = ui.radio_value(&mut osckind, "sinewave".to_string(), "SineWave");
                let _ = ui.radio_value(&mut osckind, "sawtooth".to_string(), "SawTooth");
                let _ = ui.radio_value(&mut osckind, "rectangular".to_string(), "Rectangular");
                let _ = ui.radio_value(&mut osckind, "triangular".to_string(), "Triangular");
                ui.ctx().data_mut(|d| {
                    d.insert_persisted(id, osckind.clone());
                });
                (addosc, osckind)
            })
            .inner;
        let addfile = ui.button("💾 Load File");
        let mut array_num = 5;
        let addarray = ui
            .horizontal(|ui| {
                let b = ui.button("~ Add oscillators…");
                let _ = ui.add(egui::DragValue::new(&mut array_num));
                b
            })
            .inner;
        if addosc.clicked() {
            let region = make_region(trackid, pos, osckind);
            let _ = sender.send(action::AddRegion::new(region, trackid).into());
        }
        if addfile.clicked() {
            let (file, _len) = data::generator::FilePlayerParam::new_test_file();
            //todo!
            let content = data::Content::Generator(data::Generator::FilePlayer(Arc::new(file)));
            let region = make_region(trackid, pos, "audiofile".to_string());
            let _ = sender.send(action::AddRegion::new(region, trackid).into());
        }
        if addarray.clicked() {
            // self.add_region_array(self.id, self.state.new_array_count);
        }
        (addosc, addfile, addarray)
    })
    .response
    .on_hover_text("Add new clip")
}
