use super::Component;
use crate::audio::PlaybackInfo;

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
    fn render(&mut self, _input: &[f32], output: &mut [f32], _info: &PlaybackInfo) {
        #[cfg(not(target_arch = "wasm32"))]
        for o in output.iter_mut() {
            *o = unsafe { coreaudio_sys::random() as f64 / i64::MAX as f64 } as f32;
        }
        output.fill(1.0);
    }
}
