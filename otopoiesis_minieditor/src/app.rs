use std::cell::RefCell;
use std::rc::Rc;

use otopoiesis_lang::compiler::Context;
use otopoiesis_lang::error;
use otopoiesis_lang::parser::{stringifier::Stringifier, ParseContext, *};
use otopoiesis_lang::value::RawValue;
pub struct Model {
    source: String,
    compiler: Context,
    result: Option<String>,
    eval_result: Option<String>,
}
impl Model {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            source: "".into(),
            compiler: Context::default(),
            result: None,
            eval_result: None,
        }
    }
    pub fn eval(&mut self) {
        let parsectx = ParseContextRef::new(ParseContext::default());
        let expr = parse(&self.source.clone(), parsectx.clone());
        let pc = parsectx.0.borrow_mut();

        match expr {
            Ok(e) => {
                self.result = Some(Stringifier::new(&pc, 0, e.clone()).to_string());
                let mut compiler = Context::new(parsectx.0.take());
                let root = compiler.root_env;
                self.eval_result = match compiler.eval(e, root) {
                    Ok(rv) => Some(rv.get_as_float().to_string()),
                    Err(e) => Some(format!("{:?}", e)),
                }
            }
            Err(es) => {
                self.eval_result = Some(error::report_to_string(
                    &self.source,
                    "anonymous".into(),
                    &es,
                ));
            }
        }
    }
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::panel::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("eval").clicked() {
                    self.eval();
                }
            })
        });
        egui::panel::TopBottomPanel::bottom("console")
            .default_height(50.)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    self.result.as_ref().map(|e| {
                        ui.label(e);
                    });
                    self.eval_result.as_ref().map(|e| ui.label(e.to_string()))
                })
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| ui.code_editor(&mut self.source));
        });
    }
}
