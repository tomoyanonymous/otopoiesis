//! The interpreter implementations for audio rendering.

pub const DEFAULT_BUFFER_LEN: usize = 2048;

#[derive(Clone)]
pub struct PlaybackInfo {
    pub sample_rate: u32,
    pub current_time: usize,
    pub frame_per_buffer: u64,
    pub channels: u64,
}

impl PlaybackInfo {
    pub fn get_current_realtime(&self) -> f32 {
        self.current_time as f32 / self.sample_rate as f32
    }
    pub fn rewind(&mut self) {
        self.current_time = 0;
    }
}

pub mod component;
pub use component::{get_component_for_value, Component, RangedComponent, GenericRangedComponent};
pub mod generator;
pub mod region;
pub mod renderer;
pub mod timeline;
pub mod track;
