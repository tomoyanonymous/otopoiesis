
pub struct PlaybackInfo {
    pub sample_rate: f32,
}
pub trait Component {
    fn get_input_channels(&self) -> usize;
    fn get_output_channels(&self) -> usize;
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo);
}

pub mod oscillator;
pub mod renderer;
pub mod region;