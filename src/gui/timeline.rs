use crate::gui::{Component, ComponentBase};
use crate::parameter::Parameter;
use crate::utils::AtomicRange;
use crate::*;
use std::sync::Arc;


pub struct Model {
    samples: Vec<f32>,
    pub amp: Arc<parameter::FloatParameter>,
    pub freq: Arc<parameter::FloatParameter>,

}