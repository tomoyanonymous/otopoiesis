//! GUI Definitions for parameter control and visualization for application.
//! Currently implemented on [`egui`];

pub mod parameter;
pub mod app;
pub mod generator;
pub mod menu;
pub mod region;
pub mod timeline;
pub mod track;
pub mod transport;

// pub(crate) const SAMPLES_PER_PIXEL_DEFAULT: f32 = 100.0;
pub(crate) const PIXELS_PER_SEC_DEFAULT: f32 = 100.0;

pub(crate) const TRACK_HEIGHT: f32 = 100.0;

// struct UI<'a,P,A>{

// }

// trait UI<'a,P,A>{
//     type Param;
//     type State;
//     type UI = (&'a mut Param, &'a mut State);

// }
