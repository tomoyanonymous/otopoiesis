//! The interpreter implementations for audio rendering.

use crate::data::expr::EvalEnv;

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

#[derive(Clone)]
pub struct RenderCtx {
    pub env: EvalEnv,
}

impl RenderCtx {
    pub fn new() -> Self {
        Self {
            env: EvalEnv::new(),
        }
    }
}

pub trait Component: std::fmt::Debug {
    fn get_input_channels(&self) -> u64;
    fn get_output_channels(&self) -> u64;
    fn prepare_play(&mut self, info: &PlaybackInfo);
    fn render(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        info: &PlaybackInfo,
        ctx: &mut RenderCtx,
    );
}

pub mod generator;
pub mod region;
pub mod renderer;
pub mod timeline;
pub mod track;
