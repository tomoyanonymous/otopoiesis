use std::cell::RefCell;
use std::rc::Rc;

use otopoiesis_lang::compiler::Context;
use otopoiesis_lang::parser::{stringifier::Stringifier, ParseContext, *};
pub struct Model {
    source: String,
    compiler: Context,
    result: Option<String>,
}
impl Model {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            source: "".into(),
            compiler: Context::default(),
            result: None,
        }
    }
    pub fn eval(&mut self) {
        let pc = ParseContextRef::new(ParseContext::default());
        let expr = parse(&self.source.clone(), pc.clone());
        let pc = pc.0.borrow_mut();
        self.compiler.expr_storage = pc.expr_storage.clone();
        self.compiler.interner = pc.interner.clone();
        match expr {
            Ok(e) => {
                self.result = Some(Stringifier::new(&pc, 0, e).to_string());
            }
            Err(es) => {
                self.result = Some(
                    es.iter()
                        .map(|e| e.to_string())
                        .fold("".to_string(), |acc, e| format!("{acc}\n{e}")),
                )
            }
        }
    }
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::panel::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("eval").clicked(){
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
                })
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| ui.code_editor(&mut self.source));
        });
    }
}
