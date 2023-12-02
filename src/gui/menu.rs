use script::Symbol;

use crate::action::{self, Action};

use crate::script::{param_float,builtin_fn, Expr, ExtFun, Value};

use crate::parameter::{FloatParameter, Parameter, RangedNumeric};
use std::sync::{mpsc, Arc, Mutex};
fn with_fade(region: Expr) -> Expr {
    Expr::App(
        Expr::Var(Symbol::new("fadeinout")).into(),
        vec![
            region,
            Expr::Literal(Value::Parameter(Arc::new(
                FloatParameter::new(0.4, "time_in").set_range(0.0..=0.5),
            ))),
            Expr::Literal(Value::Parameter(Arc::new(
                FloatParameter::new(0.4, "time_out").set_range(0.0..=0.5),
            ))),
        ],
    )
}
fn make_region(trackid: usize, pos: f64, _c: String) -> Expr {
    let generator = Expr::Lambda(
        vec![],
        Expr::App(
            Expr::Literal(Value::ExtFunction(ExtFun::new(builtin_fn::SineWave::new()))).into(),
            vec![
                Expr::Literal(Value::Parameter(Arc::new(param_float!(
                    440.,
                    "freq",
                    20.0..=20000.
                ))))
                .into(),
                Expr::Literal(Value::Parameter(Arc::new(param_float!(
                    1.0,
                    "amp",
                    0.0..=1.
                ))))
                .into(),
                Expr::Literal(Value::Parameter(Arc::new(param_float!(
                    0.,
                    "phase",
                    0.0..=1.0
                ))))
                .into(),
            ],
        )
        .into(),
    );
    let region = Expr::Region(
        Expr::Literal(Value::Parameter(Arc::new(param_float!(
            pos as f32,
            "start",
            0.0..=f32::INFINITY
        ))))
        .into(),
        Expr::Literal(Value::Parameter(Arc::new(param_float!(
            pos as f32 + 1.0,
            "dur",
            0.0..=f32::INFINITY
        ))))
        .into(),
        generator.into(),
        format!("region{}", trackid + 1),
    );

    // with_fade(region)
    region
}

fn make_region_file(trackid: usize, pos: f64, path: String) -> Expr {
    //todo: link parameter with let binding
    let generator = Expr::Lambda(
        vec![],
        Expr::App(
            Expr::Var("fileplayer".into()).into(),
            vec![
                Expr::Literal(Value::String(Arc::new(Mutex::new(path)))),
                Expr::Literal(Value::Parameter(Arc::new(param_float!(
                    pos as f32,
                    "file_start",
                    0.0..=f32::INFINITY
                )))),
                Expr::Literal(Value::Parameter(Arc::new(param_float!(
                    pos as f32 + 1.0,
                    "file_dur",
                    0.0..=f32::INFINITY
                )))),
            ],
        )
        .into(),
    );
    let region = Expr::Region(
        Expr::Literal(Value::Parameter(Arc::new(param_float!(
            pos as f32,
            "start",
            0.0..=f32::INFINITY
        ))))
        .into(),
        Expr::Literal(Value::Parameter(Arc::new(param_float!(
            pos as f32 + 1.0,
            "dur",
            0.0..=f32::INFINITY
        ))))
        .into(),
        generator.into(),
        format!("region{}", trackid + 1),
    );
    region
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
            //todo!
            let dir = env!("CARGO_MANIFEST_DIR");
            let path = rfd::FileDialog::new()
                .set_directory(dir)
                .pick_file()
                .map(|path| path.to_str().map(String::from))
                .flatten();
            if let Some(path_str) = path {
                let region = make_region_file(trackid, pos, path_str);
                let _ = sender.send(action::AddRegion::new(region, trackid).into());
            }
        }
        if addarray.clicked() {
            // self.add_region_array(self.id, self.state.new_array_count);
        }
        (addosc, addfile, addarray)
    })
    .response
    .on_hover_text("Add new clip")
}

pub fn add_fade_to_region(
    trackid: usize,
    region_num: usize,
    sender: &mpsc::Sender<Action>,
    ui: &mut egui::Ui,
) -> egui::Response {
    let res = ui.button("Apply Fade");
    if res.clicked() {
        let a = action::AddFadeInOut {
            track_num: trackid,
            pos: region_num,
            time_in: 0.1,
            time_out: 0.1,
        };
        let _ = sender.send(a.into());
    }
    res
}
