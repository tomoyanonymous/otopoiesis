use crate::audio::{Component, PlaybackInfo};

// use crate::parameter::UIntParameter
use crate::data::{self, Region};
use crate::utils::{AtomicRange, SimpleAtomic};
use std::ops::RangeInclusive;
use std::sync::Arc;
// 基本はオフラインレンダリング

/// Interface for offline rendering.
pub trait RangedComponent: std::fmt::Debug {
    fn get_range(&self) -> RangeInclusive<i64>;
    fn get_output_channels(&self) -> u64;
    fn render_offline(&mut self, dest: &mut [f32], sample_rate: u32, channels: u64);
}

#[derive(Debug)]
pub struct FadeModel {
    pub param: Arc<data::FadeParam>,
    pub origin: Box<Model>,
}
impl FadeModel {
    fn new(p: Arc<data::FadeParam>, origin: data::Region) -> Self {
        Self {
            param: p,
            origin: Box::new(Model::new(origin, 2)),
        }
    }
}

impl RangedComponent for FadeModel {
    fn get_range(&self) -> RangeInclusive<i64> {
        let (start, end) = self.origin.params.range.get_pair();
        start..=end
    }
    fn get_output_channels(&self) -> u64 {
        2
    }
    fn render_offline(&mut self, dest: &mut [f32], sample_rate: u32, channels: u64) {
        // resize should be the caller.
        // dest.resize(self.origin.interleaved_samples_cache.len(), 0.0);
        self.origin.render_offline(sample_rate, channels);
        let chs = self.get_output_channels() as usize;
        self.origin
            .interleaved_samples_cache
            .chunks(chs)
            .zip(dest.chunks_mut(chs))
            .enumerate()
            .for_each(|(count, (v_per_channel, o_per_channel))| {
                let in_time = self.param.time_in.load() as f64 * sample_rate as f64;
                let out_time = self.param.time_out.load() as f64 * sample_rate as f64;
                let now = count as f64;

                let len = self.origin.params.range.getrange() as f64;
                let mut gain = 1.0;
                if (0.0..=in_time).contains(&now) {
                    gain = (now as f64 / in_time).clamp(0.0, 1.0);
                }

                if (len - out_time..=len).contains(&now) {
                    gain = ((len - now) as f64 / out_time).clamp(0.0, 1.0);
                }
                if now > len {
                    gain = 0.0;
                }

                v_per_channel
                    .iter()
                    .map(|s| s * gain as f32)
                    .zip(o_per_channel.iter_mut())
                    .for_each(|(v, o)| {
                        *o = v;
                    });
            });
    }
}

#[derive(Debug)]
pub struct RegionArray(Vec<Model>);
impl RegionArray {
    pub fn new(param: &[Region]) -> Self {
        Self(param.iter().map(|p| Model::new(p.clone(), 2)).collect())
    }
}

impl RangedComponent for RegionArray {
    /// panics  if the end is earlier than the start.
    ///
    fn get_range(&self) -> RangeInclusive<i64> {
        if !self.0.is_empty() {
            let start = self.0[0].params.range.start();
            let end = self.0.last().unwrap().params.range.end();
            assert!(end >= start);
            start..=end
        } else {
            0..=0
        }
    }

    fn get_output_channels(&self) -> u64 {
        2
    }

    fn render_offline(&mut self, dest: &mut [f32], sample_rate: u32, channels: u64) {
        //todo: asynchrounous render
        self.0.iter_mut().for_each(|region| {
            let range = &region.params.range;
            let dest = &mut dest[(range.start() as usize)..(range.end() as usize)];
            region
                .interleaved_samples_cache
                .resize((range.getrange() as u64 * channels) as usize, 0.0);
            region.render_offline(sample_rate, channels);
            assert_eq!(region.interleaved_samples_cache.len(), dest.len());
            dest.copy_from_slice(&region.interleaved_samples_cache);
        });
    }
}

#[derive(Debug)]
pub struct RangedComponentDyn {
    generator: Box<dyn Component + Sync + Send>,
    range: AtomicRange<i64>,
    buffer: Vec<f32>,
}

impl RangedComponentDyn {
    pub fn new(generator: Box<dyn Component + Sync + Send>, range: AtomicRange<i64>) -> Self {
        Self {
            generator,
            range,
            buffer: vec![],
        }
    }
}

impl RangedComponent for RangedComponentDyn {
    fn get_range(&self) -> RangeInclusive<i64> {
        let (start, end) = self.range.get_pair();
        start..=end
    }

    fn get_output_channels(&self) -> u64 {
        self.generator.get_output_channels()
    }

    fn render_offline(&mut self, dest: &mut [f32], sample_rate: u32, channels: u64) {
        let info_local = PlaybackInfo {
            sample_rate,
            current_time: 0,
            frame_per_buffer: dest.len() as u64 / channels,
            channels,
        };
        self.buffer.resize(self.range.getrange() as usize, 0.0);
        let input_dummy = vec![0.0f32; 1];
        self.generator.render(&input_dummy, dest, &info_local)
    }
}

#[derive(Debug)]
pub struct TransformerModel(Box<dyn RangedComponent + Send + Sync>);

impl TransformerModel {
    fn new(filter: &data::RegionFilter, origin: data::Region) -> Self {
        let component: Box<dyn RangedComponent + Send + Sync> = match filter {
            data::RegionFilter::Gain => todo!(),
            data::RegionFilter::FadeInOut(param) => Box::new(FadeModel::new(param.clone(), origin)),
            data::RegionFilter::Reverse => todo!(),
            data::RegionFilter::Replicate(c) => Box::new(RegionArray(
                (0..c.count.load())
                    .into_iter()
                    .map(|_| Model::new(origin.clone(), 2))
                    .collect::<Vec<_>>(),
            )),
        };
        Self(component)
    }
}

#[derive(Debug)]
pub struct Model {
    pub params: data::Region,
    channels: u64,
    pub interleaved_samples_cache: Vec<f32>,
    pub content: Box<dyn RangedComponent + Send + Sync>,
    cache_completed: bool,
}

impl Model {
    pub fn new(params: data::Region, channels: u64) -> Self {
        // assert!(params.range.getrange() < params.max_size);
        let buf_size = channels as u64 * 60000; //todo!
        let content: Box<dyn RangedComponent + Send + Sync> = match &params.content {
            data::Content::Generator(g) => {
                let c = super::generator::get_component_for_generator(g);
                let ranged_component = RangedComponentDyn::new(c, params.range.clone());
                Box::new(ranged_component)
            }
            data::Content::AudioFile(_) => todo!(),
            data::Content::Transformer(filter, origin) => {
                TransformerModel::new(filter, *origin.clone()).0
            } // data::Content::Array(vec) => Box::new(RegionArray::new(vec)),
        };
        Self {
            params,
            channels,
            interleaved_samples_cache: vec![0.0; buf_size as usize],
            content,
            cache_completed: false,
        }
    }
    pub fn render_offline(&mut self, sample_rate: u32, channels: u64) {
        self.interleaved_samples_cache.resize(
            (self.params.range.getrange() as u64 * channels) as usize,
            0.0,
        );
        self.content
            .render_offline(&mut self.interleaved_samples_cache, sample_rate, channels);
        self.cache_completed = true;
    }
    pub fn contains_samples(&self, range: RangeInclusive<i64>) -> bool {
        let t_range = &self.params.range;
        let start = t_range.start();
        let end = t_range.end();
        let c1 = range.contains(&start) || range.contains(&end);
        let c2 = t_range.contains(*range.start()) && t_range.contains(*range.end());
        c1 || c2
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_region_offline_async(
    region: Model,
    info: &PlaybackInfo,
) -> std::thread::JoinHandle<Model> {
    let name = region.params.label.clone();
    let info = info.clone();

    std::thread::Builder::new()
        .name(name)
        .spawn(move || {
            let mut r = region;

            // make a temporary local copy to prevent from double-mutable borrowing
            let mut dest = r.interleaved_samples_cache.clone();
            r.content
                .render_offline(&mut dest, info.sample_rate, info.channels);

            r.cache_completed = true;
            r.interleaved_samples_cache.copy_from_slice(dest.as_slice());
            r
        })
        .expect("failed to launch thread")
}
