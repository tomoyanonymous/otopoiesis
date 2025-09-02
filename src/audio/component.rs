use std::{ops::RangeInclusive, sync::Arc};

// use script::{runtime::PlayInfo, atomic::AtomicRange};

use crate::{
    atomic::AtomicRange, data::{ConversionError, Region}
    // script::{self, Environment, EvalError, Expr, Value},
};

use super::{PlaybackInfo};

pub mod mimium_component;

pub trait Component {
    fn get_input_channels(&self) -> u64;
    fn get_output_channels(&self) -> u64;
    fn prepare_play(&mut self, info: &PlaybackInfo);
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo);
}


/// Interface for offline rendering.
pub trait RangedComponent {
    fn get_range(&self) -> RangeInclusive<f64>;
    fn get_output_channels(&self) -> u64;
    fn get_sample_cache(&self) -> &[f32];
    fn get_sample_cache_mut(&mut self) -> &mut [f32];
    fn render_offline(&mut self, sample_rate: f64, channels: u64);
    fn get_cache_len(&self, sample_rate: f64) -> usize {
        let range = self.get_range();
        let len_sec = range.end() - range.start();
        (len_sec * sample_rate) as usize
    }
    fn get_sample(&self, time: f64, sample_rate: u32) -> Option<f64> {
        if self.get_range().contains(&time) {
            self.get_sample_cache()
                .get((sample_rate as f64 * (time - self.get_range().start())) as usize)
                .map(|s| *s as f64)
        } else {
            None
        }
    }
    fn get_default_playback_info(&self, sample_rate: f64, channels: u64) -> PlaybackInfo {
        let dur = self.get_range().end() - self.get_range().start();
        let numsamples = (dur * sample_rate as f64).ceil() as u64;
        PlaybackInfo {
            sample_rate,
            current_time: 0,
            frame_per_buffer: numsamples,
            channels,
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_region_offline_async(
    mut region: Box<dyn RangedComponent + Send + Sync>,
    info: &PlaybackInfo,
) -> std::thread::JoinHandle<Box<dyn RangedComponent + Send + Sync>> {
    let name = format!("regionrender{}", rand::random::<u64>());
    let info = info.clone();

    std::thread::Builder::new()
        .name(name)
        .spawn(move || {
            use crate::audio::PlayInfo;

            region.render_offline(info.get_samplerate(), info.get_channels());
            region
        })
        .expect("failed to launch thread")
}

//convert any generator component into region

pub struct GenericRangedComponent {
    generator: Box<dyn Component + Sync + Send>,
    range: AtomicRange,
    buffer: Vec<f32>,
}

impl GenericRangedComponent {
    pub fn new(generator: Box<dyn Component + Sync + Send>, range: AtomicRange) -> Self {
        Self {
            generator,
            range,
            buffer: vec![],
        }
    }
}

impl RangedComponent for GenericRangedComponent {
    fn get_range(&self) -> RangeInclusive<f64> {
        self.range.start() as f64..=self.range.end() as f64
    }

    fn get_output_channels(&self) -> u64 {
        self.generator.get_output_channels()
    }

    fn render_offline(&mut self, sample_rate: f64, channels: u64) {
        let len = (self.range.getrange() as f64 * sample_rate as f64) as usize * channels as usize;
        let info_local = PlaybackInfo {
            sample_rate,
            current_time: 0,
            frame_per_buffer: len as u64 / channels,
            channels,
        };
        self.buffer.resize(len, 0.0);
        let input_dummy = vec![0.0f32; 1];
        self.generator.prepare_play(&info_local);
        let mut dest = self.buffer.clone();
        // let dest = self.get_sample_cache_mut();
        self.generator.render(&input_dummy, &mut dest, &info_local);
        self.buffer = dest;
    }

    fn get_sample(&self, _time: f64, _sample_rate: u32) -> Option<f64> {
        todo!()
    }

    fn get_sample_cache(&self) -> &[f32] {
        &self.buffer
    }
    fn get_sample_cache_mut(&mut self) -> &mut [f32] {
        &mut self.buffer
    }
}