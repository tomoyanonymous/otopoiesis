//gui component built on top of nannou framework.
// it has handler for relative mouse event position relative to bounding box.
// no more needed?


use nannou::prelude::*;
pub mod timeline;
pub mod region;
pub mod track;
pub mod generator;

#[derive(PartialEq)]
pub enum MouseState {
    None,
    Hover,
    Clicked(Point2), //saves position when clicked. useful for make drag function
}
pub use nannou::geom::Rect;
//mousepos stores global position in window.
//user can get local position through get_local_mouse_pos in Component trait.
pub struct ComponentBase {
    bound: Rect,
    draw_bound: bool,
    mousestate: MouseState,
    pub mousepos: Point2,
}
impl ComponentBase {
    pub fn new(bound: Rect) -> Self {
        Self {
            bound,
            draw_bound: false,
            mousestate: MouseState::None,
            mousepos: Point2::new(0., 0.),
        }
    }
}
pub trait Component {
    //3 functions user must implement.
    fn get_base_component_mut(&mut self) -> &mut ComponentBase;
    fn get_base_component(&self) -> &ComponentBase;

    fn draw(&self, ctx: &Draw);

    //optional function user can override behaviour.

    fn mouse_moved(&mut self, _pos: Point2) {}
    fn mouse_pressed(&mut self, _mouse: MouseButton) {}
    fn mouse_dragged(&mut self, _origin: Point2, _current: Point2) {}
    fn mouse_released(&mut self, _mouse: MouseButton) {}

    //getters
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
    fn is_mouse_on(&self) -> bool {
        let base = self.get_base_component();
        base.bound.contains_point(base.mousepos.to_array())
    }
    fn is_mouse_pressed(&self) -> bool {
        match self.get_base_component().mousestate {
            MouseState::Clicked(_) => true,
            _ => false,
        }
    }

    //called by actual app.
    fn draw_raw(&self, app: &App, frame: & Frame) {
        let mut draw = app.draw();
        let base = self.get_base_component();
        let bound = self.get_bounding_box();

        // draw = draw.x_y(-frame.rect().w() * 0.5, frame.rect().h() * 0.5);

        if base.draw_bound {
            draw.rect()
                .x_y(bound.x(), bound.y())
                .w_h(bound.w(), bound.h())
                .no_fill()
                .stroke_color(rgb(255., 0., 0.))
                .stroke_weight(1.0);
        }
        draw = draw.scissor(bound);
        self.draw(&draw);
        draw.to_frame(app, &frame).ok();
    }
    fn set_draw_boundary(&mut self, b: bool) {
        self.get_base_component_mut().draw_bound = b;
    }

    fn mouse_moved_raw(&mut self, pos: Point2) {
        let is_mouseon = self.is_mouse_on();
        let base = self.get_base_component_mut();
        base.mousepos = pos;
        match base.mousestate {
            MouseState::None => {
                if is_mouseon {
                    base.mousestate = MouseState::Hover;
                }
            }
            MouseState::Clicked(origin) => self.mouse_dragged(origin, pos),
            _ => {}
        }
        self.mouse_moved(pos);
    }

    fn mouse_pressed_raw(&mut self, mouse: MouseButton) {
        if let MouseButton::Left = mouse {
            self.get_base_component_mut().mousestate = MouseState::Clicked(self.get_mouse_pos());
        }
        self.mouse_pressed(mouse)
    }

    fn mouse_released_raw(&mut self, mouse: MouseButton) {
        let is_mouseon = self.is_mouse_on();
        let base = self.get_base_component_mut();

        if let MouseButton::Left = mouse {
            if is_mouseon {
                base.mousestate = MouseState::Hover;
            } else {
                base.mousestate = MouseState::None;
            }
        }
        self.mouse_released(mouse);
    }
}
