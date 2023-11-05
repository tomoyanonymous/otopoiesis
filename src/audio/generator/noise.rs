use super::Component;
use crate::audio::{PlaybackInfo, RenderCtx};
use rand;
#[derive(Clone, Debug)]
pub struct Noise {}
impl Component for Noise {
    fn get_input_channels(&self) -> u64 {
        0
    }
    fn get_output_channels(&self) -> u64 {
        2
    }

    fn prepare_play(&mut self, _info: &PlaybackInfo) {}
    fn render(
        &mut self,
        _input: &[f32],
        output: &mut [f32],
        _info: &PlaybackInfo,
        _ctx: &mut RenderCtx,
    ) {
        for o in output.iter_mut() {
            *o = rand::random();
        }
        output.fill(1.0);
    }
}
