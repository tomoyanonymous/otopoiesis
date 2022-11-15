pub struct PlaybackInfo {
    pub sample_rate: f32,
    pub current_time: usize,
}

impl PlaybackInfo {
    pub fn get_current_realtime(&self) -> f32 {
        self.current_time as f32 / self.sample_rate
    }
}

pub trait Component {
    fn get_input_channels(&self) -> usize;
    fn get_output_channels(&self) -> usize;
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo);
}

pub mod oscillator;
pub mod region;
pub mod renderer;
