use crate::audio::{self, Component, PlaybackInfo};
use crate::data;
use std::sync::Arc;

pub struct Model {
    param: Arc<data::Project>,
    tracks: Vec<super::track::Model>, // regions: Vec<audio::region::Region<>>
    tmp_buffer: Vec<f32>,
}

impl Model {
    pub fn new(param: Arc<data::Project>) -> Self {
        let tracks = param
            .tracks
            .iter()
            .map(|t| super::track::Model::new(Arc::clone(&t), 2))
            .collect::<Vec<_>>();
        let tmp_buffer = vec![0.0; 3];
        Self {
            param: Arc::clone(&param),
            tracks,
            tmp_buffer,
        }
    }
}
impl Component for Model {
    fn get_input_channels(&self) -> u64 {
        0
    }
    fn get_output_channels(&self) -> u64 {
        2
    }
    fn prepare_play(&mut self, info: &PlaybackInfo) {
        assert_eq!(self.tracks.len(), self.param.tracks.len());
        self.tmp_buffer
            .resize((info.channels * info.frame_per_buffer) as usize, 0.0);
        for track in self.tracks.iter_mut() {
            track.prepare_play(info);
        }
    }
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        assert_eq!(output.len(), self.tmp_buffer.len());
        for track in self.tracks.iter_mut() {
            track.render(input, self.tmp_buffer.as_mut_slice(), info);
            output
                .iter_mut()
                .zip(self.tmp_buffer.iter())
                .for_each(|(out, tmp)| *out += *tmp);
        }
    }
}
