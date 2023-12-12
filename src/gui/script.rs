use egui::InnerResponse;
use script::EnvTrait;

use crate::gui::parameter::slider_from_parameter;
use crate::script::{Environment, EvalError, Expr, Type, Value};
use std::sync::Arc;

pub fn eval_ui_val(v: &Value, ui: &mut egui::Ui) -> egui::InnerResponse<Result<Value, EvalError>> {
    let response = match v {
        Value::Number(n) => ui.label(format!("{n}")),
        Value::Parameter(p) => slider_from_parameter(p, false, ui),
        Value::None => todo!(),
        Value::String(s) => {
            if let Ok(ref mut s) = s.try_lock() {
                ui.text_edit_singleline(s as &mut String)
            } else {
                ui.label("failed to lock thread for text ui")
            }
        }
        Value::Array(vec, _t) => {
            ui.push_id(ui.next_auto_id(), |ui| {
                ui.group(|ui| {
                    vec.iter().for_each(|v| {
                        let _ = eval_ui_val(v, ui);
                    })
                })
                .response
            })
            .inner
        }
        Value::Function(_, _) => todo!(),
        Value::Closure(_ids, env, body) => {
            ui.group(|ui| eval_ui(body, env.clone(), ui).response).inner
        }
        Value::ExtFunction(f) => ui.label(f.get_name()),
        Value::Track(env, rgs, _t) => {
            ui.group(|ui| {
                ui.label("Track");
                eval_ui(rgs, env.clone(), ui)
            })
            .response
        }
        Value::Region(env, start, dur, content, label, _t) => {
            ui.group(|ui| {
                ui.label(format!("Region-{}", label));
                let _s = eval_ui(start, env.clone(), ui);
                let _d = eval_ui(dur, env.clone(), ui);
                eval_ui(content, env.clone(), ui)
            })
            .response
        }
        Value::Project(_env, _sr, _content) => {
            todo!()
        }
    };
    egui::InnerResponse {
        inner: Ok(v.clone()),
        response: response,
    }
}

pub fn eval_ui(
    e: &Expr,
    env: Arc<Environment>,
    ui: &mut egui::Ui,
) -> egui::InnerResponse<Result<Value, EvalError>> {
    match e {
        Expr::Literal(v) => eval_ui_val(v, ui),
        Expr::Array(vec) => {
            ui.push_id(ui.next_auto_id(), |ui| {
                ui.group(|ui| {
                    ui.label("array");
                    let res = vec
                        .iter()
                        .map(|e| eval_ui(e, env.clone(), ui).inner)
                        .try_collect::<Vec<_>>()?;
                    Ok(Value::Array(res, Type::Unknown))
                })
            })
            .inner
        }
        Expr::Var(v) => {
            if let Some(mut val) = env.lookup(&v) {
                eval_ui_val(&mut val, ui)
            } else {
                InnerResponse::new(
                    Err(EvalError::NotFound),
                    ui.label(format!("{:?} not found", v)),
                )
            }
        }
        Expr::Let(_, _, _) => todo!(),
        Expr::App(callee, args) => ui.group(|ui| {
            let _ = eval_ui(callee, env.clone(), ui);
            ui.group(|ui| {
                let arr = args
                    .iter()
                    .map(|a| eval_ui(a, env.clone(), ui).inner)
                    .try_collect::<Vec<_>>()?;
                Ok(Value::Array(arr, Type::Unknown))
            })
            .inner
        }),
        Expr::Lambda(_ids, body) => eval_ui(body, env, ui),
        Expr::Track(rg) => eval_ui(rg, env, ui),
        Expr::Region(start, dur, content, _) => ui.group(|ui| {
            eval_ui(start, env.clone(), ui)
                .inner
                .and(eval_ui(dur, env.clone(), ui).inner)
                .and(eval_ui(content, env, ui).inner)
        }),
        Expr::Nop => todo!(),
        Expr::BinOp(_, _, _) => todo!(),
        Expr::AppExt(_, _) => todo!(),
        Expr::Paren(_) => todo!(),
        Expr::WithAttribute(_, _) => todo!(),
        Expr::Project(_, _) => todo!(),
        
    }
}
