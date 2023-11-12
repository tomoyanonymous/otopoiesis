use crate::action::{self, Action};
use crate::data::{
    self,
    script::{Expr, Type, Value},
    Content, OscillatorFun, Region,
};

use crate::utils::{atomic, AtomicRange};
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
            Expr::Literal(region).into(),
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
        let mut osckind_str = "sinewave".to_string();
        let addosc = ui
            .horizontal(|ui| {
                let addosc = ui.button("~ Add oscillator");
                let _ = ui.radio_value(&mut osckind_str, "sinewave".to_string(), "SineWave");
                let _ = ui.radio_value(&mut osckind_str, "sawtooth".to_string(), "SawTooth");
                let _ = ui.radio_value(&mut osckind_str, "rectangular".to_string(), "Rectangular");
                let _ = ui.radio_value(&mut osckind_str, "triangular".to_string(), "Triangular");
                addosc
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
            let region = make_region(trackid, pos, osckind_str);
            let _ = sender.send(action::AddRegion::new(region, trackid).into());
        }
        if addfile.clicked() {
            let (file, _len) = data::generator::FilePlayerParam::new_test_file();
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
