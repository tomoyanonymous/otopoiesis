use crate::audio::{Component, PlaybackInfo};
use crate::data;
use std::sync::Arc;
pub struct Model {
    param: Arc<data::Track>,
    regions: Vec<super::region::Model>,
}

impl Model {
    pub fn new(param: Arc<data::Track>, channels: u64) -> Self {
        let regions = param
            .0
            .iter()
            .map(|region| {
                super::region::Model::new(
                    Arc::clone(&region),
                    channels,
                    super::oscillator::get_component_for_generator(&region.generator),
                )
            })
            .collect::<Vec<_>>();

        Self { param, regions }
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
        assert_eq!(self.regions.len(), self.param.0.len());
        for region in self.regions.iter_mut() {
            region.prepare_play(info);
        }
    }
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        //後に入ってるリージョンで基本は上書きする
        //channel is tekitou
        for (count, out_per_channel) in output.chunks_mut(2 as usize).enumerate() {
            let now = (info.current_time + count) as u64;

            out_per_channel.iter_mut().enumerate().for_each(|(ch, s)| {
                //順にリージョンを読んでいくので、重なってる場合は後の要素のやつが上書きする形になる
                for region in self.regions.iter() {
                    if region.params.range.contains(now) {
                        let read_point = ((now - region.params.range.start()) * 2) as usize;
                        let out = region.interleaved_samples_cache[read_point + ch];
                        *s = out;
                    }else{
                        *s = 0.0;
                    }
                }
            });
        }
    }
}
