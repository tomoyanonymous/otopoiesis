use crate::audio::{Component, PlaybackInfo};

// use crate::parameter::UIntParameter
use crate::utils::AtomicRange;
use std::sync::Arc;

// modifierが後で追加されたりする。生成用にComponentを持っている？
// Buffer by Bufferで再生するという時にどうタイミングを合わせるか？
// Region{range:0..=2000,2,vec![0.0]}

pub struct Model {
    range: Arc<AtomicRange>,
    channels: usize,
    interleaved_samples_cache: Vec<f32>,
    pub generator: Box<dyn Component + Send>,
    cache_completed: bool,
}

// pub trait EditableRegion {
//     fn set_start(newv: u64) {}
// }

impl Model {
    pub fn new(
        range: Arc<AtomicRange>,
        channels: usize,
        generator: Box<dyn Component + Send>,
    ) -> Self {
        let buf_size = channels as u64 * (range.getrange());
        Self {
            range,
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
    pub fn render_offline_async(&mut self, info: PlaybackInfo) {
        todo!()
    }
}

impl Component for Model {
    fn get_input_channels(&self) -> usize {
        0
    }
    fn get_output_channels(&self) -> usize {
        self.channels
    }
    //info.current_time contains exact sample from the beggining at a head of the buffer.
    fn prepare_play(&mut self, info: &PlaybackInfo) {
        self.render_offline(info);
    }
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        for (count, out_per_channel) in output.chunks_mut(self.channels).enumerate() {
            let now = (info.current_time + count) as u64;
            let in_range = self.range.contains(now);
            let has_cache = self.cache_completed;
            if in_range && has_cache {
                let read_point = ((now - self.range.start()) * 2) as usize;
                out_per_channel
                    .iter_mut()
                    .enumerate()
                    .for_each(|(ch, s)| *s = self.interleaved_samples_cache[read_point + ch]);
            }
        }
    }
}
