use super::oscillator::GeneratorComponent;
use crate::audio::{Component, PlaybackInfo};
use std::ops::RangeBounds;
use std::ops::RangeInclusive;
use std::thread;
// use crate::parameter::UIntParameter
use crate::utils::AtomicRange;
use std::sync::Arc;

// modifierが後で追加されたりする。生成用にComponentを持っている？
// Buffer by Bufferで再生するという時にどうタイミングを合わせるか？
// Region{range:0..=2000,2,vec![0.0]}

// impl PartialOrd<AtomicU64> for AtomicU64{
//     fn partial_cmp(&self, other: &AtomicU64) -> Option<std::cmp::Ordering> {
//         use std::cmp::Ordering;
//         let self_v = self.load(std::sync::atomic::Ordering::Relaxed);
//         let other_v = other.load(std::sync::atomic::Ordering::Relaxed);
//         self_v.partial_cmp(&other_v)
//     }

// }



pub struct Region {
    range: Arc<AtomicRange>,
    channels: usize,
    interleaved_samples_cache: Vec<f32>,
    cache_completed: bool,
}

// pub trait EditableRegion {
//     fn set_start(newv: u64) {}
// }

impl Region {
    pub fn new(range: Arc<AtomicRange>, channels: usize) -> Self {
        let buf_size = channels as u64 * (range.getrange());
        Self {
            range,
            channels,
            interleaved_samples_cache: vec![0.0; buf_size as usize],
            cache_completed: false,
        }
    }
    pub fn render_offline<E: GeneratorComponent>(
        &mut self,
        generator: &mut E,
        info: &PlaybackInfo,
    ) {
        for (_count, out) in self
            .interleaved_samples_cache
            .chunks_mut(self.channels)
            .enumerate()
        {
            for (_ch, o) in out.iter_mut().enumerate() {
                generator.render_sample(o, info)
            }
        }
        self.cache_completed = true;
    }
    pub fn render_offline_async<E: GeneratorComponent + Send + Sync>(
        &mut self,
        generator: E,
        info: PlaybackInfo,
    ) {
    }
}

impl Component for Region {
    fn get_input_channels(&self) -> usize {
        0
    }
    fn get_output_channels(&self) -> usize {
        self.channels
    }
    //info.current_time contains exact sample from the beggining at a head of the buffer.

    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        for (count, o) in output.chunks_mut(self.channels).enumerate() {
            let now = (info.current_time + count) as u64;
            for (ch, s) in o.iter_mut().enumerate() {
                *s = if self.range.contains(&now) {
                    if self.cache_completed {
                        let read_point = (now - self.range.start()) as i64;
                        assert_eq!(read_point >= 0, true);
                        self.interleaved_samples_cache[read_point as usize + ch]
                    } else {
                        // self.generate()
                        0.0
                    }
                } else {
                    0.0
                }
            }
        }
    }
}
