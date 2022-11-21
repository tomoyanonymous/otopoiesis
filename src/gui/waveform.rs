use crate::gui::{Component, ComponentBase};
use crate::parameter::Parameter;
use crate::*;
use std::sync::Arc;

pub struct Model {
    samples: Vec<f32>,
    pub osc_params: Arc<oscillator::SharedParams>,
    pub region_params: Arc<AtomicRange>,
    horizontal_scale: f32,
    amp_tmp: f32,
    freq_tmp: f32,
    base: ComponentBase,
}
pub fn range_to_bound(horizontal_scale: f32, range: (u64, u64)) -> Rect {
    Rect::from_x_y_w_h(
        range.0 as f32 * horizontal_scale,
        50.,
        range.1 as f32 * horizontal_scale,
        400.,
    )
}
impl Model {
    pub fn update_samples(&mut self) {
        let mut phase = 0.0f32;
        let len = self.samples.len();
        for s in self.samples.iter_mut() {
            *s = phase.sin();
            let twopi = std::f32::consts::PI * 2.0;
            //とりあえず、440Hzで1周期分ということで
            let ratio = self.osc_params.freq.get() / 440.0;
            let increment = ratio * twopi / len as f32;
            phase = (phase + increment) % twopi;
        }
    }

    pub fn new(
        bound: Rect,
        osc_params: Arc<oscillator::SharedParams>,
        region_params: Arc<AtomicRange>,
    ) -> Self {
        let size = 512;
        let samples = vec![0f32; size];
        let horizontal_scale = 0.01;
        let amp_tmp = osc_params.amp.get();
        let freq_tmp = osc_params.freq.get();
        let rect = range_to_bound(horizontal_scale, region_params.get_pair());
        let base = ComponentBase::new(rect);
        let mut res = Self {
            samples,
            osc_params,
            region_params,
            horizontal_scale,
            amp_tmp,
            freq_tmp,
            base,
        };
        res.update_samples();
        res
    }
    pub fn get_current_amp(&self) -> f32 {
        self.osc_params.amp.get().abs()
    }
    fn is_on_start_handle(&self, cursor: Point2) -> bool {
        let left = self.get_bounding_box().left();
        ((left - 10.)..(left + 10.)).contains(&cursor.x)
    }
    fn is_on_end_handle(&self, cursor: Point2) -> bool {
        let right = self.get_bounding_box().right();
        ((right - 10.)..(right + 10.)).contains(&cursor.x)
    }
    fn dragg_start(&mut self, origin: Point2, current: Point2) {}
}

impl Component for Model {
    fn get_base_component_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
    fn get_base_component(&self) -> &ComponentBase {
        &self.base
    }
    fn mouse_moved(&mut self, _pos: Point2) {}
    fn mouse_dragged(&mut self, origin: Point2, current: Point2) {
        let bound = self.get_base_component_mut().bound;

        if self.is_on_start_handle(self.get_mouse_pos()) {
            let shift = (bound.left() + current.x) / self.horizontal_scale;
            self.region_params.set_start(shift as u64);
            self.get_base_component_mut().bound.x.start = current.x;
        } else if self.is_on_end_handle(self.get_mouse_pos()) {
            let shift = (bound.left() + bound.w() + current.x) / self.horizontal_scale;
            self.region_params.set_end(shift as u64);
            self.get_base_component_mut().bound.x.end = current.x;
        } else {
            let params = &self.osc_params;
            params.amp.set(self.amp_tmp + (current.y - origin.y) * 0.01);
            params
                .freq
                .set(self.freq_tmp + (current.x - origin.x) * 10.);
            self.update_samples();
        }
    }
    fn mouse_released(&mut self, _mouse: MouseButton) {
        let params = &self.osc_params;

        self.amp_tmp = params.amp.get();
        self.freq_tmp = params.freq.get();
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
            let y = *s * 100.0 * self.get_current_amp();
            nannou::geom::pt2(x, y)
        }));
        if self.is_on_start_handle(self.get_mouse_pos()) {
            ctx.line()
                .weight(2.0)
                .start(bound.top_left())
                .end(bound.bottom_left());
        }
        if self.is_on_end_handle(self.get_mouse_pos()) {
            ctx.line()
                .weight(2.0)
                .start(bound.top_right())
                .end(bound.bottom_right());
        }
        if self.is_mouse_pressed() {
            ctx.ellipse().radius(10.).xy(self.get_local_mouse_pos());
        }
        let str = format!(
            "x:{:.2},y:{:.2}",
            self.get_local_mouse_pos().x,
            self.get_local_mouse_pos().y
        );
        ctx.text(str.as_str())
            .xy(self.get_local_mouse_pos() + Vec2::new(0., 20.));
        let params = &self.osc_params;

        let str2 = format!("amp:{:.2},freq:{:.2}", params.amp.get(), params.freq.get());
        ctx.text(str2.as_str())
            .xy(self.get_local_mouse_pos() + Vec2::new(0., -20.));
        let range = self.region_params.get_pair();
        let str3 = format!("range:{:?}", range);
        ctx.text(str3.as_str())
            .xy(self.get_local_mouse_pos() + Vec2::new(0., -40.));
    }

    fn mouse_pressed(&mut self, _mouse: MouseButton) {}
}
