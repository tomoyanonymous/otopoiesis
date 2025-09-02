use std::cell::RefCell;

use crate::parameter::FloatParameter;
use crate::parameter::{Parameter, RangedNumeric};
use mimium_lang::{
    ast::{Expr, Literal},
    function,
    interner::{ToSymbol, TypeNodeId},
    interpreter::Value,
    log, numeric,
    plugin::{ExtClsInfo, SysPluginSignature, SystemPlugin, SystemPluginFnType},
    runtime::vm::{Machine, ReturnCode},
    string_t,
    types::{PType, Type},
};
enum UIData {
    Track {
        params: Vec<FloatParameter>,
        content: Vec<Box<UIData>>,
    },
    Clip(Vec<FloatParameter>),
    Slider(FloatParameter),
}
trait UIElement {
    fn consume_stack<T: UIElement>(&mut self, elems: &[T]) {}
}

pub struct OtopoiesisPlugin {
    pub slider_storage: Vec<FloatParameter>,
}

impl OtopoiesisPlugin {
    pub fn make_slider(&mut self, v: &[(Value, TypeNodeId)]) -> Value {
        assert_eq!(v.len(), 4);
        let (name, target, min, max) = match (
            v[0].0.clone(),
            v[1].0.clone(),
            v[2].0.clone(),
            v[3].0.clone(),
        ) {
            (Value::String(name), Value::Code(e), Value::Number(min), Value::Number(max)) => {
                (name, e, min, max)
            }
            _ => {
                log::error!("invalid argument");
                return Value::Number(0.0);
            }
        };
        let param =
            FloatParameter::new(target, name.to_string()).set_range(min as f32..=max as f32);
        self.slider_storage.push(param);
        let idx = self.slider_storage.len() - 1;
        Value::Code(
            Expr::Apply(
                Expr::Var("get_param".to_symbol()).into_id_without_span(),
                vec![Expr::Literal(Literal::Float(RefCell::new(idx as f64))).into_id_without_span()],
            )
            .into_id_without_span(),
        )
    }
    pub fn get_slider(&self, vm: &mut Machine) -> ReturnCode {
        let slider_idx = Machine::get_as::<f64>(vm.get_stack(0)) as usize;
        match self.slider_storage.get(slider_idx) {
            Some(s) => {
                vm.set_stack(0, Machine::to_value(s.get()));
            }
            None => {
                log::error!("invalid slider index");
                return 0;
            }
        };
        1
    }
}

impl SystemPlugin for OtopoiesisPlugin {
    fn on_init(&mut self, _machine: &mut Machine) -> ReturnCode {
        0
    }
    fn after_main(&mut self, _machine: &mut Machine) -> ReturnCode {
        0
    }
    fn gen_interfaces(&self) -> Vec<SysPluginSignature> {
        todo!()
    }
}
