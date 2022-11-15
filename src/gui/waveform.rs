use crate::gui::{Component, ComponentBase};
use crate::*;
use std::sync::{atomic::Ordering, Arc};

pub struct Model {
    samples: Vec<f32>,
    pub amp: Arc<parameter::FloatParameter>,
    amp_tmp: f32,
    base: ComponentBase,
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
        let amp = Arc::new(parameter::FloatParameter::new(1., 0.0..1.0, "amplitude"));
        let amp_tmp = amp.get();
        let base = ComponentBase::new(bound);
        Self {
            samples,
            amp,
            amp_tmp,
            base,
        }
    }
    pub fn get_current_amp(&self) -> f32 {
        self.amp.get().abs()
    }
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
        self.amp.set(self.amp_tmp + (current.y - origin.y) * 0.01);
    }
    fn mouse_released(&mut self, _mouse: MouseButton) {
        self.amp_tmp = self.amp.get();
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
    }

    fn mouse_pressed(&mut self, _mouse: MouseButton) {}

    fn get_bounding_box(&self) -> nannou::geom::Rect {
        return self.get_base_component().bound;
    }

    fn get_mouse_pos(&self) -> Point2 {
        self.get_base_component().mousepos
    }

    fn get_local_mouse_pos(&self) -> Point2 {
        let base = self.get_base_component();
        base.mousepos - base.bound.xy()
    }

    fn draw_raw(&self, app: &App, frame: Frame) {
        let mut draw = app.draw();
        let base = self.get_base_component();
        let bound = self.get_bounding_box();

        // draw = draw.x_y(-frame.rect().w() * 0.5, frame.rect().h() * 0.5);

        if base.draw_bound {
            draw.rect()
                .x_y(0., 0.)
                .w_h(bound.w(), bound.h())
                .no_fill()
                .stroke_color(rgb(255., 0., 0.))
                .stroke_weight(1.0);
        }
        draw = draw.scissor(bound);
        self.draw(&draw);
        draw.to_frame(app, &frame).ok();
    }

    fn is_mouse_on(&self) -> bool {
        let base = self.get_base_component();
        base.bound.contains_point(base.mousepos.to_array())
    }

    fn is_mouse_pressed(&self) -> bool {
        match self.get_base_component().mousestate {
            gui::MouseState::Clicked(_) => true,
            _ => false,
        }
    }

    fn set_draw_boundary(&mut self, b: bool) {
        self.get_base_component_mut().draw_bound = b;
    }

    fn mouse_moved_raw(&mut self, pos: Point2) {
        let is_mouseon = self.is_mouse_on();
        let base = self.get_base_component_mut();
        base.mousepos = pos;
        match base.mousestate {
            gui::MouseState::None => {
                if is_mouseon {
                    base.mousestate = gui::MouseState::Hover;
                }
            }
            gui::MouseState::Clicked(origin) => self.mouse_dragged(origin, pos),
            _ => {}
        }
        self.mouse_moved(pos);
    }

    fn mouse_pressed_raw(&mut self, mouse: MouseButton) {
        if let MouseButton::Left = mouse {
            self.get_base_component_mut().mousestate = gui::MouseState::Clicked(self.get_mouse_pos());
        }
        self.mouse_pressed(mouse)
    }

    fn mouse_released_raw(&mut self, mouse: MouseButton) {
        let is_mouseon = self.is_mouse_on();
        let base = self.get_base_component_mut();

        if let MouseButton::Left = mouse {
            if is_mouseon {
                base.mousestate = gui::MouseState::Hover;
            } else {
                base.mousestate = gui::MouseState::None;
            }
        }
        self.mouse_released(mouse);
    }
}
