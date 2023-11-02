use crate::action::{self, Action};
use crate::data::{self, Content, OscillatorFun, Region};

use crate::utils::{atomic, AtomicRange};
use std::sync::{mpsc, Arc};

fn make_region(trackid: usize, pos: f64, c: Content) -> Region {
    let region_param = data::Region::new(
        AtomicRange::<f64>::from(pos..pos + 1.0),
        c,
        format!("region{}", trackid + 1),
    );
    data::Region::with_fade(region_param)
}

pub fn add_region_button(
    trackid: usize,
    pos: f64,
    sender: &mpsc::Sender<Action>,
    ui: &mut egui::Ui,
) -> egui::Response {
    ui.menu_button("+", |ui| {
        let osckindid = ui.auto_id_with("osckind");
        let mut osckind = ui
            .data_mut(|d| {
                let kind = d.get_persisted::<OscillatorFun>(osckindid);
                kind
            })
            .unwrap_or_default();
        let addosc = ui
            .horizontal(|ui| {
                let addosc = ui.button("~ Add oscillator");
                let _ = ui.radio_value(&mut osckind, OscillatorFun::SineWave, "Sinewave");
                let _ = ui.radio_value(
                    &mut osckind,
                    OscillatorFun::SawTooth(Arc::new(atomic::Bool::new(true))),
                    "Sawtooth",
                );
                let _ = ui.radio_value(&mut osckind, OscillatorFun::Triangular, "Triangular");
                let _ = ui.radio_value(
                    &mut osckind,
                    OscillatorFun::Rectanglular(Arc::new(atomic::F32::new(0.5))),
                    "Rectangular",
                );
                addosc
            })
            .inner;
        ui.data_mut(|d| d.insert_persisted(osckindid, osckind.clone()));
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
            let content = data::Content::Generator(data::Generator::Oscillator(
                osckind,
                Arc::new(data::OscillatorParam::default()),
            ));
            let region = make_region(trackid, pos, content);
            let _ = sender.send(action::AddRegion::new(region, trackid).into());
        }
        if addfile.clicked() {
            let (file, _len) = data::generator::FilePlayerParam::new_test_file();
            let content = data::Content::Generator(data::Generator::FilePlayer(Arc::new(file)));
            let region = make_region(trackid, pos, content);
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
