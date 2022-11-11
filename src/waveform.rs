// use core::slice::SlicePattern;
use super::component;

use nannou::prelude::*;

pub struct Model {
    samples: Vec<f32>,
    amp: f32,
    amp_tmp: f32,
    base: component::ComponentBase,
}

impl Model {
    pub fn new(bound: nannou::geom::Rect) -> Self {
        let size = 512;
        let mut samples = vec![0f32; size];
        let mut phase = 0.0f32;
        for s in samples.iter_mut() {
            *s = phase.sin();
            let twopi = std::f32::consts::PI * 2.0;
            let increment = twopi / size as f32;
            phase = (phase + increment) % twopi;
        }
        let amp = 1.0;
        let amp_tmp = 0.;
        let base = component::ComponentBase::new(bound);
        Self {
            samples,
            amp,
            amp_tmp,
            base,
        }
    }
}

impl component::Component for Model {
    fn get_base_component_mut(&mut self) -> &mut component::ComponentBase {
        &mut self.base
    }
    fn get_base_component(&self) -> &component::ComponentBase {
        &self.base
    }
    fn mouse_moved(&mut self, _pos: Point2) {}
    fn mouse_dragged(&mut self, origin: Point2, current: Point2) {
        self.amp_tmp = (current.y - origin.y) * 0.01;
    }
    fn mouse_released(&mut self, _mouse: MouseButton) {
        self.amp += self.amp_tmp;
        self.amp_tmp = 0.;
    }
    fn draw(&self, ctx: &Draw) {
        let bound = self.get_bounding_box();

        let line = ctx.polyline();
        line.points(self.samples.iter().enumerate().map(|(i, s)| {
            let x = nannou::math::map_range(
                i as f32,
                0.,
                self.samples.len() as f32,
                bound.left(),
                bound.right(),
            );
            let y = *s * 100.0 * (self.amp + self.amp_tmp);
            nannou::geom::pt2(x, y)
        }));

        if self.is_mouse_pressed() {
            ctx.ellipse().radius(10.).xy(self.get_local_mouse_pos());
        }
        let str = format!(
            "x:{:.2},y:{:.2}",
            self.get_local_mouse_pos().x,
            self.get_local_mouse_pos().y
        );
        ctx.text(str.as_str()).xy(self.get_local_mouse_pos()+Vec2::new(0.,20.));
    }
}
