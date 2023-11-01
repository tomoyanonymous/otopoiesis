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
       ui.ctx().memory_mut(|memory| {
        ui.output(.)
            memory.data.insert_persisted("osckind".into(), OscillatorFun::SineWave)
        });
        let osckind =     ui.ctx().memory_mut(|memory| {
            memory.data.get_persisted_mut_or_default("osckind".into())
        });
        let addosc = ui
            .horizontal(|ui| {
   
                let addosc = ui.button("~ Add oscillator");
                let _ = ui.radio_value(osckind, OscillatorFun::SineWave, "Sinewave");
                let _ = ui.radio_value(
                    osckind,
                    OscillatorFun::SawTooth(Arc::new(atomic::Bool::new(true))),
                    "Sawtooth",
                );
                let _ = ui.radio_value(osckind,  OscillatorFun::Triangular, "Triangular");
                let _ = ui.radio_value(
                    osckind,
                    OscillatorFun::Rectanglular(Arc::new(atomic::F32::new(0.5))),
                    "Rectangular",
                );
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
            let content = data::Content::Generator(data::Generator::Oscillator(
                ui.ctx()
                    .data(|d| d.get_temp("osckind".into()))
                    .unwrap(),
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
