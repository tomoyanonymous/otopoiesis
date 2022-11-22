use crate::gui;
use crate::parameter::Parameter;
use crate::utils::AtomicRange;
use crate::*;
use std::sync::Arc;

pub struct Model {
    time: u64,
    base: gui::ComponentBase,
}

impl Model {
    pub fn new(bound: gui::Rect) -> Self {
        Self {
            time: 0,
            base: gui::ComponentBase::new(bound),
        }
    }
}
