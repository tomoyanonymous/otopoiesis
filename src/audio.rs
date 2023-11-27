//! The interpreter implementations for audio rendering.

pub const DEFAULT_BUFFER_LEN: usize = 2048;

#[derive(Clone,Copy)]
pub struct PlaybackInfo {
    pub sample_rate: f64,
    pub current_time: u64,
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
    pub fn boxed(self)->Box<dyn PlayInfo+Send+Sync>{
        let b:Box<dyn PlayInfo+Send+Sync> = Box::new(self);
        b
    }
}
use crate::script::runtime::PlayInfo;
impl PlayInfo for PlaybackInfo{
    fn get_current_time_in_sample(&self)->u64 {
        self.current_time 
    }

    fn get_samplerate(&self)->f64 {
        self.sample_rate 
    }

    fn increment_time(&mut self) {
        self.current_time+=1;
    }

    fn get_channels(&self)->u64 {
        self.channels
    }

    fn get_frame_per_buffer(&self)->u64 {
        self.frame_per_buffer
    }
}

pub mod component;
pub use component::{get_component_for_value, Component, RangedComponent, RangedComponentDyn};
pub mod generator;
pub mod region;
pub mod renderer;
pub mod timeline;
pub mod track;
