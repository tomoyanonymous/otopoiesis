use crate::audio::{Component, PlaybackInfo};

// use crate::parameter::UIntParameter
use crate::data;
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};
// 基本はオフラインレンダリング

#[derive(Debug)]
pub struct Model {
    pub params: Arc<data::Region>,
    channels: u64,
    pub interleaved_samples_cache: Vec<f32>,
    pub generator: Box<dyn Component + Send + Sync>,
    cache_completed: bool,
}

// pub trait EditableRegion {
//     fn set_start(newv: u64) {}
// }

impl Model {
    pub fn new(params: Arc<data::Region>, channels: u64) -> Self {
        // assert!(params.range.getrange() < params.max_size);
        let buf_size = channels as u64 * 60000; //todo!
        let generator = super::generator::get_component_for_generator(&params.generator);
        Self {
            params,
            channels,
            interleaved_samples_cache: vec![0.0; buf_size as usize],
            generator,
            cache_completed: false,
        }
    }
    pub fn render_offline(&mut self, info: &PlaybackInfo) {
        let info_local = PlaybackInfo {
            sample_rate: info.sample_rate,
            current_time: 0,
            frame_per_buffer: self.interleaved_samples_cache.len() as u64 / info.channels,
            channels: info.channels,
        };
        let dummy_input = [0.0];
        self.generator.prepare_play(&info_local);
        self.generator.render(
            &dummy_input,
            self.interleaved_samples_cache.as_mut_slice(),
            &info_local,
        );
        self.cache_completed = true;
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

pub fn render_region_offline_async<'a>(
    region: Model,
    info: &PlaybackInfo,
) -> std::thread::JoinHandle<Model> {
    let name = region.params.label.clone();
    let info = info.clone();

    let res = std::thread::Builder::new()
        .name(name.clone())
        .spawn(move || {
            let mut r = region;
            let info_local = PlaybackInfo {
                sample_rate: info.sample_rate,
                current_time: 0,
                frame_per_buffer: r.interleaved_samples_cache.len() as u64 / info.channels,
                channels: info.channels,
            };
            let dummy_input = [0.0];
            // make a temporary local copy to prevent from double-mutable borrowing
            let mut dest = r.interleaved_samples_cache.clone();
            r.generator.prepare_play(&info_local);
            r.generator
                .render(&dummy_input, dest.as_mut_slice(), &info_local);
            r.cache_completed = true;
            r.interleaved_samples_cache.copy_from_slice(dest.as_slice());

            return r;
        })
        .expect("failed to launch thread");
    res
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
