use crate::action::{self, Action};
use crate::data;
use crate::script::{Environment, Expr, Type, Value};

use crate::parameter::{FloatParameter, Parameter, RangedNumeric};
use std::sync::{mpsc, Arc};
fn with_fade(region: Value) -> Value {
    Value::Closure(
        vec![],
        Arc::new(Environment::new()),
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
fn make_region(trackid: usize, pos: f64, c: String) -> Value {
    let generator = Value::new_lazy(Expr::App(
        Expr::Literal(Value::ExtFunction(c)).into(),
        vec![
            Expr::Literal(Value::Parameter(Arc::new(
                FloatParameter::new(440., "freq").set_range(10.0..=20000.),
            ))),
            Expr::Literal(Value::Parameter(Arc::new(
                FloatParameter::new(1.0, "amp").set_range(0.0..=1.0),
            ))),
            Expr::Literal(Value::Parameter(Arc::new(
                FloatParameter::new(0.0, "phase").set_range(0.0..=1.0),
            ))),
        ],
    ));
    let region = Value::Region(
        pos,
        pos + 1.0,
        generator.into(),
        format!("region{}", trackid + 1),
        Type::Unknown,
    );

    with_fade(region)
}

fn make_region_file(trackid: usize, pos: f64, path: String) -> Value {
    let generator = Value::new_lazy(Expr::App(
        Expr::Literal(Value::ExtFunction("fileplayer".to_string())).into(),
        vec![Expr::Literal(Value::String(path))],
    ));
    let region = Value::Region(
        pos,
        pos + 1.0,
        generator.into(),
        format!("region{}", trackid + 1),
        Type::Unknown,
    );
    with_fade(region)
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

            let region = make_region_file(trackid, pos, file.path);
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
