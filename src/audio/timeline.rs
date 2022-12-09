use crate::audio::{Component, PlaybackInfo};
use crate::data;
use std::sync::Arc;
#[derive(Debug)]
pub struct Model {
    param: Arc<data::Project>,
    transport: Arc<data::Transport>,
    tracks: Vec<super::track::Model>, // regions: Vec<audio::region::Region<>>
    tmp_buffer: Vec<f32>,
}

impl Model {
    pub fn new(project: Arc<data::Project>, transport: Arc<data::Transport>) -> Self {
        let tracks = Self::get_new_tracks(project.as_ref());
        let tmp_buffer = vec![0.0; 3];
        Self {
            param: Arc::clone(&project),
            transport: Arc::clone(&transport),
            tracks,
            tmp_buffer,
        }
    }
    fn get_new_tracks(project: &data::Project) -> Vec<super::track::Model> {
        project
            .tracks
            .lock()
            .unwrap()
            .iter()
            .map(|t| match t {
                data::Track::Regions(r) => super::track::Model::new(r.clone(), 2),
                data::Track::Generator(_) => todo!(),
                data::Track::Transformer() => todo!(),
            })
            .collect::<Vec<_>>()
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
        self.tracks = Self::get_new_tracks(self.param.as_ref());
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
