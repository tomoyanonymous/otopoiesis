use std::sync::Arc;

use super::Component;
use crate::audio::PlaybackInfo;
use crate::parameter::{FloatParameter, Parameter};
#[derive(Clone, Debug)]
pub struct Constant(pub Arc<FloatParameter>);
impl Component for Constant {
    fn get_input_channels(&self) -> u64 {
        0
    }
    fn get_output_channels(&self) -> u64 {
        2
    }

    fn prepare_play(&mut self, _info: &PlaybackInfo) {}
    fn render(&mut self, _input: &[f32], output: &mut [f32], _info: &PlaybackInfo) {
        output.fill(self.0.get());
    }
}
