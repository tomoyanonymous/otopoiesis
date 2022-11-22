use crate::audio::{self, Component, PlaybackInfo};
use std::sync::Arc;
use crate::data;

pub struct Model {
    param: Arc<data::Project>,
    // regions: Vec<audio::region::Region<>>
}

impl Component for Model {
    fn get_input_channels(&self) -> usize {
        0
    }
    fn get_output_channels(&self) -> usize {
        2
    }
    fn prepare_play(&mut self, info: &PlaybackInfo) {}
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo) {}
}
