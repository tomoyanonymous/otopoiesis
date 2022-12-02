use crate::audio::{Component, PlaybackInfo};

// use crate::parameter::UIntParameter
use crate::data;
use std::ops::RangeInclusive;
use std::sync::Arc;
// modifierが後で追加されたりする。生成用にComponentを持っている？
// Buffer by Bufferで再生するという時にどうタイミングを合わせるか？
// Region{range:0..=2000,2,vec![0.0]}

pub struct Model {
    pub params: Arc<data::Region>,
    channels: u64,
    pub interleaved_samples_cache: Vec<f32>,
    pub generator: Box<dyn Component + Send>,
    cache_completed: bool,
}

// pub trait EditableRegion {
//     fn set_start(newv: u64) {}
// }

impl Model {
    pub fn new(
        params: Arc<data::Region>,
        channels: u64,
        generator: Box<dyn Component + Send>,
    ) -> Self {
        // assert!(params.range.getrange() < params.max_size);
        let buf_size = channels as u64 * 60000; //todo!
        Self {
            params,
            channels,
            interleaved_samples_cache: vec![0.0; buf_size as usize],
            generator,
            cache_completed: false,
        }
    }
    pub fn render_offline(&mut self, info: &PlaybackInfo) {
        let dummy_input = [0.0];
        self.generator.render(
            &dummy_input,
            self.interleaved_samples_cache.as_mut_slice(),
            info,
        );

        self.cache_completed = true;
    }
    pub fn render_offline_async(&mut self, _info: PlaybackInfo) {
        todo!()
    }
    pub fn contains_samples(&self, range: RangeInclusive<u64>) -> bool {
        let t_range = &self.params.range;
        let start = t_range.start();
        let end = t_range.end();
        let c1 = range.contains(&start) || range.contains(&end);
        let c2 = t_range.contains(*range.start()) && t_range.contains(*range.end());
        c1 || c2
    }
}

impl Component for Model {
    fn get_input_channels(&self) -> u64 {
        0
    }
    fn get_output_channels(&self) -> u64 {
        self.channels
    }
    //info.current_time contains exact sample from the beggining at a head of the buffer.
    fn prepare_play(&mut self, info: &PlaybackInfo) {
        self.render_offline(info);
    }
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        for (count, out_per_channel) in output.chunks_mut(self.channels as usize).enumerate() {
            let now = (info.current_time + count) as u64;
            let in_range = self.params.range.contains(now);
            let has_cache = self.cache_completed;
            if in_range && has_cache {
                let read_point = ((now - self.params.range.start()) * 2) as usize;
                out_per_channel
                    .iter_mut()
                    .enumerate()
                    .for_each(|(ch, s)| *s = self.interleaved_samples_cache[read_point + ch]);
            }
        }
    }
}
