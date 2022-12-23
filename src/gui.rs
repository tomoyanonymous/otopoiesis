//! GUI Definitions for parameter control and visualization for application. 
//! Currently implemented on [`egui`].

pub mod timeline;
pub mod region;
pub mod track;
pub mod generator;
pub mod app;
pub mod transport;

pub(crate) const SAMPLES_PER_PIXEL_DEFAULT:f32= 100.0;

pub(crate) const TRACK_HEIGHT:f32= 100.0;
