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

impl gui::Component for Model {
    fn get_base_component_mut(&mut self) -> &mut gui::ComponentBase {
        &mut self.base
    }
    fn get_base_component(&self) -> &gui::ComponentBase {
        &self.base
    }
    fn mouse_moved(&mut self, _pos: Point2) {}
    fn mouse_dragged(&mut self, origin: Point2, current: Point2) {}
    fn mouse_released(&mut self, _mouse: MouseButton) {}
    fn draw(&self, ctx: &Draw) {}

    fn mouse_pressed(&mut self, _mouse: MouseButton) {}
}
