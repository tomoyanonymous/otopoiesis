use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, mpsc};

use crate::atomic::{self, SimpleAtomic};
use crate::data::{self, ProbeMap, Region, SliderId, SliderMap, Track};
use crate::parameter::FloatParameter;
use crate::parameter::{Parameter, RangedNumeric};
use mimium_lang::code;
use mimium_lang::plugin::SystemPluginMacroType;
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
use slotmap::Key;
pub type ProjectChannel = mpsc::Sender<data::Project>;
pub struct OtopoiesisPlugin {
    pub param_stack: Vec<SliderId>,
    track_stack: Vec<Track>,
    region_stack: Vec<Region>,
    probe_map: Rc<RefCell<ProbeMap>>,
    slider_map: Vec<Arc<FloatParameter>>,
    project_channel: ProjectChannel,
    shared_time: Arc<atomic::U64>,
}

impl OtopoiesisPlugin {
    pub fn new(project_sender: ProjectChannel) -> Self {
        Self {
            param_stack: vec![],
            track_stack: vec![],
            region_stack: vec![],
            probe_map: Default::default(),
            slider_map: Default::default(),
            project_channel: project_sender,
            shared_time: Arc::new(atomic::U64::from(0)), //lazily initialized
        }
    }

    fn mimium_getnow(&mut self, vm: &mut Machine) -> ReturnCode {
        let time = self.shared_time.load() as f64;
        vm.set_stack(0, Machine::to_value(time));
        1
    }
    fn mimium_getsamplerate(&mut self, vm: &mut Machine) -> ReturnCode {
        let sr = 44100.0; // TODO: get from project
        vm.set_stack(0, Machine::to_value(sr));
        1
    }
    fn make_slider(&mut self, v: &[(Value, TypeNodeId)]) -> Value {
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
        let cell = match target.to_expr_ref() {
            Expr::Literal(Literal::Float(n)) => n,
            _ => {
                log::error!("invalid target for slider");
                return Value::Number(0.0);
            }
        };
        let param =
            FloatParameter::new(cell.clone(), name.to_string()).set_range(min as f32..=max as f32);
        self.slider_map.push(Arc::new(param));
        let sliderid = self.slider_map.len() - 1;
        self.param_stack.push(sliderid);
        Value::Code(
            Expr::Apply(
                Expr::Var("get_param".to_symbol()).into_id_without_span(),
                vec![
                    Expr::Literal(Literal::Float(Arc::new(atomic::F64::new(sliderid as f64))))
                        .into_id_without_span(),
                ],
            )
            .into_id_without_span(),
        )
    }
    pub fn get_slider(&mut self, vm: &mut Machine) -> ReturnCode {
        let slider_idx = Machine::get_as::<f64>(vm.get_stack(0)) as u64;
        let s = self.slider_map.get(slider_idx as usize);
        match s {
            Some(s) => {
                // let v = s.get();
                vm.set_stack(0, Machine::to_value(1.0f64));
            }
            None => {
                log::error!("invalid slider index");
                return 0;
            }
        };
        1
    }
    pub fn make_project(&mut self, v: &[(Value, TypeNodeId)]) -> Value {
        assert_eq!(v.len(), 2);
        let (name, content) = match (v[0].0.clone(), v[1].0.clone()) {
            (Value::String(name), Value::Code(expr)) => (name, expr),
            _ => {
                log::error!("invalid argument");
                return Value::Number(0.0);
            }
        };

        let mut project = data::Project::new(name.to_string(), 44100);
        for slider in self.param_stack.drain(..) {
            project.parameters.push(slider);
        }
        project.slider_map = self.slider_map.clone();
        self.shared_time = project.current_time.clone();
        self.project_channel
            .send(project)
            .expect("could not send project");
        Value::Code(content)
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
        let getnowf: SystemPluginFnType<Self> = Self::mimium_getnow;
        let get_now =
            SysPluginSignature::new("_mimium_getnow", getnowf, function!(vec![], numeric!()));
        let getsrf: SystemPluginFnType<Self> = Self::mimium_getsamplerate;
        let get_sr = SysPluginSignature::new(
            "_mimium_getsamplerate",
            getsrf,
            function!(vec![], numeric!()),
        );
        let sliderf: SystemPluginMacroType<Self> = Self::make_slider;
        let make_slider = SysPluginSignature::new_macro(
            "Slider",
            sliderf,
            function!(
                vec![string_t!(), numeric!(), numeric!(), numeric!()],
                Type::Code(Type::Primitive(PType::Numeric).into_id()).into_id()
            ),
        );
        let getsliderf: SystemPluginFnType<Self> = Self::get_slider;
        let get_slider = SysPluginSignature::new(
            "get_param",
            getsliderf,
            function!(vec![numeric!()], numeric!()),
        );

        let makeprojectf: SystemPluginMacroType<Self> = Self::make_project;
        let thunkt = code!(function!(vec![], numeric!()));
        let make_project = SysPluginSignature::new_macro(
            "Project",
            makeprojectf,
            function!(vec![string_t!(), thunkt], thunkt),
        );
        vec![get_now, get_sr, make_slider, get_slider, make_project]
    }
}
