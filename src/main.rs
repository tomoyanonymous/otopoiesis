use crate::utils::AtomicRange;
use nannou::prelude::*;
use nannou_egui::{egui, Egui};
use otopoiesis::*;
use parameter::{FloatParameter, Parameter};
use serde_json;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use crate::audio::{
    renderer::{Renderer, RendererBase},
    Component,
};
use crate::data;
use crate::gui;

fn main() {
    nannou::app(model)
        .event(event)
        .update(update)
        .view(view)
        .run();
}
struct Model {
    app: Arc<Mutex<data::AppModel>>,
    project_str: String,
    code_compiled: serde_json::Result<Arc<data::Project>>,
    audio: Renderer<audio::timeline::Model>,
    egui: Egui,
    is_played: bool,
}

impl Model {
    pub fn new(egui: Egui) -> Self {
        let region_len = 60000;
        let sample_rate = 44100 as u64;
        let osc_param = Arc::new(data::OscillatorParam {
            amp: FloatParameter::new(1.0, 0.0..=1.0, "amp"),
            freq: FloatParameter::new(440.0, 20.0..=20000.0, "freq"),
            phase: FloatParameter::new(0.0, 0.0..=6.3, "phase"),
        });
        let region_param = Arc::new(data::Region {
            range: AtomicRange::new(1000, 50000),
            max_size: AtomicU64::from(60000),
            generator: Arc::new(data::Generator::Oscillator(Arc::clone(&osc_param))),
            filters: vec![],
            label: String::from("region0"),
        });
        let project = Arc::new(data::Project {
            sample_rate: AtomicU64::from(sample_rate),
            tracks: Arc::new(Mutex::new(vec![data::Track(Arc::new(Mutex::new(vec![
                Arc::clone(&region_param),
            ])))])),
        });
        let transport = Arc::new(data::Transport::new());
        let app = Arc::new(Mutex::new(data::AppModel::new(
            Arc::clone(&transport),
            Arc::new(data::GlobalSetting {}),
            Arc::clone(&project),
        )));
        let json = serde_json::to_string_pretty(&project);
        let json_str = json.unwrap_or("failed to parse".to_string());
        let mut timeline =
            audio::timeline::Model::new(Arc::clone(&project), Arc::clone(&transport));
        // let sinewave = audio::oscillator::SineModel::new(Arc::clone(&osc_param));
        // let mut region =
        //     audio::region::Model::new(Arc::clone(&region_param), 2, Box::new(sinewave));
        let info = audio::PlaybackInfo {
            sample_rate: sample_rate as u32,
            current_time: 0,
            channels: 2,
            frame_per_buffer: 512,
        };
        timeline.prepare_play(&info);

        let renderer = audio::renderer::create_renderer(
            timeline,
            Some(44100),
            Some(512),
            Arc::clone(&transport.time),
        );

        Self {
            audio: renderer,
            app: Arc::clone(&app),
            project_str: json_str,
            code_compiled: Ok(project),
            egui,
            is_played: false,
        }
    }
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size(1200, 800)
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
    let egui = Egui::from_window(&app.window(window_id).unwrap());

    let mut res = Model::new(egui);
    res.audio.prepare_play();
    res.audio.pause();
    // res.audio.play();
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

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    model.is_played = model.audio.is_playing();

    let mut app_gui = gui::app::Model {
        param: Arc::clone(&model.app),
    };
    app_gui.show_ui(&ctx);

    egui::panel::SidePanel::right("JSON viewer")
        .default_width(300.)
        .min_width(300.)
        .max_width(1920.)
        .resizable(true)
        .show(&ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let editor = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut model.project_str).code_editor(),
                );
                let mut app = model.app.lock().unwrap();
                if editor.gained_focus() {
                    let json = serde_json::to_string_pretty(&app.project);
                    let json_str = json.unwrap_or("failed to parse".to_string());
                    model.project_str = json_str;
                }
                if editor.lost_focus() {
                    let proj = serde_json::from_str::<Arc<data::Project>>(&model.project_str);
                    model.code_compiled = proj;
                    if let Ok(proj) = &model.code_compiled {
                        app.project = Arc::clone(proj);
                    }
                }

                if let Err(err) = &model.code_compiled {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("failed to evaluate json:{}", err.to_string()),
                    );
                }

                // ui.code_editor(model.
            });
        });
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(SKYBLUE);

    model.egui.draw_to_frame(&frame).unwrap();
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

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    let is_cmd_down = if cfg!(target_os = "macos") {
        app.keys.mods.logo()
    } else {
        app.keys.mods.ctrl()
    };
    
    match key {
        Key::Space => {
            if model.audio.is_playing() {
                model.audio.pause();
            } else {
                model.audio.rewind();
                model.audio.prepare_play();
                model.audio.play();
            }
        }
        Key::Z => {
            if is_cmd_down & !app.keys.mods.shift() {
                println!("undo");
                let history = &mut model.app.lock().unwrap().history;
                let _ = history.undo(&mut ()).unwrap();
            }
            if is_cmd_down & app.keys.mods.shift() {
                println!("redo");
                let history = &mut model.app.lock().unwrap().history;
                let _ = history.redo(&mut ()).unwrap();
            }
        }
        _ => {}
    }
}

fn key_released(_app: &App, _model: &mut Model, _key: Key) {}

fn mouse_moved(_app: &App, model: &mut Model, pos: Point2) {}

fn mouse_pressed(_app: &App, model: &mut Model, button: MouseButton) {}

fn mouse_released(_app: &App, model: &mut Model, button: MouseButton) {}

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
