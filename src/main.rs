use nannou::prelude::*;
use std::sync::Arc;

use parameter::{FloatParameter, Parameter};
use utils::AtomicRange;

mod audio;
mod gui;
mod parameter;
mod utils;

use audio::{
    oscillator, region,
    renderer::{Renderer, RendererBase},
    Component,
};

use gui::waveform;
use gui::Component as UiComponent;

fn main() {
    nannou::app(model)
        .event(event)
        .update(update)
        .view(view)
        .run();
}
struct Model {
    wave_ui: waveform::Model,
    audio: Renderer<region::Model>,
}

impl Model {
    pub fn new() -> Self {
        let area = nannou::geom::Rect::from_x_y_w_h(-400., 0., 400., 600.);

        let sinewave_params = Arc::new(oscillator::SharedParams {
            amp: FloatParameter::new(1.0, 0.0..=1.0, "amp"),
            freq: FloatParameter::new(440.0, 20.0..=20000.0, "freq"),
        });
        let sinewave = oscillator::SineModel::new(Arc::clone(&sinewave_params));

        let range_params = Arc::new(AtomicRange::new(1000, 50000));
        let mut region = region::Model::new(Arc::clone(&range_params), 2, Box::new(sinewave));
        let info = audio::PlaybackInfo {
            sample_rate: 44100,
            current_time: 0,
        };
        region.prepare_play(&info);

        let waveui = waveform::Model::new(area, sinewave_params, range_params);
        let renderer = audio::renderer::create_renderer(region, Some(44100), Some(512));

        Self {
            wave_ui: waveui,
            audio: renderer,
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 800)
        .event(window_event)
        .raw_event(raw_window_event)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .mouse_moved(mouse_moved)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .mouse_wheel(mouse_wheel)
        .mouse_entered(mouse_entered)
        .mouse_exited(mouse_exited)
        .touch(touch)
        .touchpad_pressure(touchpad_pressure)
        .moved(window_moved)
        .resized(window_resized)
        .hovered_file(hovered_file)
        .hovered_file_cancelled(hovered_file_cancelled)
        .dropped_file(dropped_file)
        .focused(window_focused)
        .unfocused(window_unfocused)
        .closed(window_closed)
        .build()
        .unwrap();

    let mut res = Model::new();
    res.audio.prepare_play();
    res.audio.play();
    res
}

fn event(_app: &App, _model: &mut Model, event: Event) {
    match event {
        Event::WindowEvent {
            id: _,
            //raw: _,
            simple: _,
        } => {}
        Event::DeviceEvent(_device_id, _event) => {}
        Event::Update(_dt) => {}
        Event::Suspended => {}
        Event::Resumed => {}
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.wave_ui.set_draw_boundary(true);
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(SKYBLUE);

    model.wave_ui.draw_raw(app, frame);
}

fn window_event(_app: &App, _model: &mut Model, event: WindowEvent) {
    match event {
        KeyPressed(_key) => {}
        KeyReleased(_key) => {}
        ReceivedCharacter(_char) => {}
        MouseMoved(_pos) => {}
        MousePressed(_button) => {}
        MouseReleased(_button) => {}
        MouseEntered => {}
        MouseExited => {}
        MouseWheel(_amount, _phase) => {}
        Moved(_pos) => {}
        Resized(_size) => {}
        Touch(_touch) => {}
        TouchPressure(_pressure) => {}
        HoveredFile(_path) => {}
        DroppedFile(_path) => {}
        HoveredFileCancelled => {}
        Focused => {}
        Unfocused => {}
        Closed => {}
    }
}

fn raw_window_event(_app: &App, _model: &mut Model, _event: &nannou::winit::event::WindowEvent) {}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            model.audio.pause();
            model.audio.rewind();
            model.audio.prepare_play();
            model.audio.play();
        }
        _ => {}
    }
}

fn key_released(_app: &App, _model: &mut Model, _key: Key) {}

fn mouse_moved(_app: &App, model: &mut Model, pos: Point2) {
    model.wave_ui.mouse_moved_raw(pos);
}

fn mouse_pressed(_app: &App, model: &mut Model, button: MouseButton) {
    model.wave_ui.mouse_pressed_raw(button);
}

fn mouse_released(_app: &App, model: &mut Model, button: MouseButton) {
    model.wave_ui.mouse_released_raw(button);
}

fn mouse_wheel(_app: &App, _model: &mut Model, _dt: MouseScrollDelta, _phase: TouchPhase) {}

fn mouse_entered(_app: &App, _model: &mut Model) {}

fn mouse_exited(_app: &App, _model: &mut Model) {}

fn touch(_app: &App, _model: &mut Model, _touch: TouchEvent) {}

fn touchpad_pressure(_app: &App, _model: &mut Model, _pressure: TouchpadPressure) {}

fn window_moved(_app: &App, _model: &mut Model, _pos: Point2) {}

fn window_resized(_app: &App, _model: &mut Model, _dim: Vec2) {}

fn window_focused(_app: &App, _model: &mut Model) {}

fn window_unfocused(_app: &App, _model: &mut Model) {}

fn window_closed(_app: &App, _model: &mut Model) {}

fn hovered_file(_app: &App, _model: &mut Model, _path: std::path::PathBuf) {}

fn hovered_file_cancelled(_app: &App, _model: &mut Model) {}

fn dropped_file(_app: &App, _model: &mut Model, _path: std::path::PathBuf) {}
