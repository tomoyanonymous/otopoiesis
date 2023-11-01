//! GUI Definitions for parameter control and visualization for application.
//! Currently implemented on [`egui`].

use crate::parameter::{FloatParameter, Parameter};

pub mod app;
pub mod generator;
pub mod region;
pub mod timeline;
pub mod track;
pub mod transport;
pub mod menu;

// pub(crate) const SAMPLES_PER_PIXEL_DEFAULT: f32 = 100.0;
pub(crate) const PIXELS_PER_SEC_DEFAULT: f32 = 100.0;

pub(crate) const TRACK_HEIGHT: f32 = 100.0;

fn slider_from_parameter(param: &FloatParameter, is_log: bool) -> egui::Slider<'_> {
    let range = &param.range;
    egui::Slider::from_get_set(
        *range.start() as f64..=*range.end() as f64,
        |v: Option<f64>| {
            if let Some(n) = v {
                param.set(n as f32);
            }
            param.get() as f64
        },
    )
    .logarithmic(is_log)
}

// struct UI<'a,P,A>{
    
// }

// trait UI<'a,P,A>{
//     type Param;
//     type State;
//     type UI = (&'a mut Param, &'a mut State);


// }