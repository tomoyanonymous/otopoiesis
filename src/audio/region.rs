use crate::audio::{Component, PlaybackInfo};

// use crate::parameter::UIntParameter
use crate::data::{self, Region};
use crate::utils::{AtomicRange, SimpleAtomic};
use std::ops::RangeInclusive;
use std::sync::Arc;
// 基本はオフラインレンダリング

/// Interface for offline rendering.
pub trait RangedComponent: std::fmt::Debug {
    fn get_range(&self) -> RangeInclusive<f64>;
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
    fn get_range(&self) -> RangeInclusive<f64> {
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
        assert_eq!(self.origin.interleaved_samples_cache.len(), dest.len());
        let chs = self.get_output_channels() as usize;
        let in_time = (self.param.time_in.load() as f64 * sample_rate as f64) as usize;
        let out_time = (self.param.time_out.load() as f64 * sample_rate as f64) as usize;

        let slice = &self.origin.interleaved_samples_cache[0..dest.len()];
        dest.copy_from_slice(slice);

        if in_time > 0 {
            self.origin.interleaved_samples_cache[0..in_time]
                .chunks(chs)
                .zip(dest[0..in_time].chunks_mut(chs))
                .enumerate()
                .for_each(|(count, (v_per_channel, o_per_channel))| {
                    let now = count as u32;

                    let gain = now as f64 / in_time as f64;

                    v_per_channel
                        .iter()
                        .zip(o_per_channel.iter_mut())
                        .for_each(|(v, o)| {
                            *o = (*v as f64 * gain) as f32;
                        });
                });
        }
        let len_arr = self.origin.interleaved_samples_cache.len();
        if out_time > 0 {
            self.origin.interleaved_samples_cache[out_time..len_arr]
                .rchunks(chs)
                .zip(dest[(len_arr - out_time)..len_arr].rchunks_mut(chs))
                .enumerate()
                .for_each(|(count, (v_per_channel, o_per_channel))| {
                    let now = count as u32;
                    let gain = if out_time > 0 {
                        now as f64 / out_time as f64
                    } else {
                        1.0
                    };
                    v_per_channel
                        .iter()
                        .zip(o_per_channel.iter_mut())
                        .for_each(|(v, o)| {
                            *o = (*v as f64 * gain) as f32;
                        });
                });
        }
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
    fn get_range(&self) -> RangeInclusive<f64> {
        if !self.0.is_empty() {
            let start = self.0[0].params.range.start();
            let end = self.0.last().unwrap().params.range.end();
            assert!(end >= start);
            start..=end
        } else {
            0.0..=0.0
        }
    }

    fn get_output_channels(&self) -> u64 {
        2
    }

    fn render_offline(&mut self, dest: &mut [f32], sample_rate: u32, channels: u64) {
        //todo: asynchrounous render
        self.0.iter_mut().for_each(|region| {
            let range = &region.params.range;
            let scale_to_index = |x: f64| (x * sample_rate as f64) as usize * channels as usize;
            let dest = &mut dest[scale_to_index(range.start())..scale_to_index(range.end())];
            region
                .interleaved_samples_cache
                .resize(scale_to_index(range.getrange()), 0.0); // no need?
            region.render_offline(sample_rate, channels);
            assert_eq!(region.interleaved_samples_cache.len(), dest.len());
            dest.copy_from_slice(&region.interleaved_samples_cache);
        });
    }
}

#[derive(Debug)]
pub struct RangedComponentDyn {
    generator: Box<dyn Component + Sync + Send>,
    range: AtomicRange<f64>,
    // buffer: Vec<f32>,
}

impl RangedComponentDyn {
    pub fn new(generator: Box<dyn Component + Sync + Send>, range: AtomicRange<f64>) -> Self {
        Self {
            generator,
            range,
            // buffer: vec![],
        }
    }
}

impl RangedComponent for RangedComponentDyn {
    fn get_range(&self) -> RangeInclusive<f64> {
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
        // self.buffer.resize(
        //     (self.range.getrange() * sample_rate as f64) as usize * channels as usize,
        //     0.0,
        // );
        let input_dummy = vec![0.0f32; 1];
        self.generator.prepare_play(&info_local);
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
    _channels: u64,
    pub interleaved_samples_cache: Vec<f32>,
    pub content: Box<dyn RangedComponent + Send + Sync>,
    cache_completed: bool,
}

impl Model {
    pub fn new(params: data::Region, channels: u64) -> Self {
        // assert!(params.range.getrange() < params.max_size);

        let content: Box<dyn RangedComponent + Send + Sync> = match &params.content {
            data::Content::Generator(g) => {
                let c = super::generator::get_component_for_generator(g);
                let ranged_component = RangedComponentDyn::new(c, params.range.clone());
                Box::new(ranged_component)
            }
            data::Content::Transformer(filter, origin) => {
                TransformerModel::new(filter, *origin.clone()).0
            }
        };
        Self {
            params,
            _channels: channels,
            interleaved_samples_cache: vec![],
            content,
            cache_completed: false,
        }
    }
    pub fn render_offline(&mut self, sample_rate: u32, channels: u64) {
        self.interleaved_samples_cache.resize(
            (self.params.range.getrange() * sample_rate as f64) as usize * channels as usize,
            0.0,
        );
        self.content
            .render_offline(&mut self.interleaved_samples_cache, sample_rate, channels);
        self.cache_completed = true;
    }
    pub fn contains_samples(&self, range: RangeInclusive<f64>) -> bool {
        let t_range = &self.params.range;
        let start = t_range.start();
        let end = t_range.end();
        let c1 = range.contains(&start) || range.contains(&end);
        let c2 = t_range.contains(*range.start()) && t_range.contains(*range.end());
        c1 || c2
    }
}

#[cfg(not(feature = "web"))]

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
            r.render_offline(info.sample_rate, info.channels);
            r
        })
        .expect("failed to launch thread")
}

#[cfg(test)]
mod test {
    use crate::parameter::Parameter;

    use super::*;

    fn gen_sinewave(
        arr: &mut [f32],
        osc_param: &data::OscillatorParam,
        phase: &mut f32,
        sample_rate: u32,
        channel: u32,
    ) {
        arr.chunks_mut(channel as usize)
            .enumerate()
            .for_each(|(_count, o_per_channel)| {
                // let now = count as f64 / sample_rate as f64;
                let twopi = std::f32::consts::PI * 2.0;
                o_per_channel.iter_mut().enumerate().for_each(|(ch, o)| {
                    if ch == 0 {
                        *o = phase.sin() * osc_param.amp.get();
                    } else {
                        *o = 0.0;
                    }
                });
                *phase = (*phase + twopi * osc_param.freq.get() / (sample_rate as f32)) % twopi;
            });
    }
    fn gen_constant(arr: &mut [f32], channel: u32) {
        arr.chunks_mut(channel as usize)
            .for_each(|out_per_channel| {
                out_per_channel.iter_mut().enumerate().for_each(|(_ch, o)| {
                    *o = 1.0;
                })
            });
    }
    fn apply_fadeinout(
        arr: &mut [f32],
        in_time: f64,
        out_time: f64,
        sample_rate: u32,
        channel: u32,
    ) {
        let in_sample = (in_time * sample_rate as f64) as usize;
        let out_sample = (out_time * sample_rate as f64) as usize;

        //do not consider the case of fade handle crosses
        debug_assert!(
            in_sample + out_sample <= arr.len(),
            "fadein time + fadeout time should be less than region length"
        );
        if in_sample > 0 {
            arr[0..in_sample]
                .chunks_mut(channel as usize)
                .enumerate()
                .for_each(|(i, o_per_channel)| {
                    let gain = i as f64 / in_sample as f64;
                    o_per_channel.iter_mut().for_each(|o| {
                        *o = ((*o as f64) * gain) as f32;
                    });
                });
        }
        let last = arr.len();
        if out_sample > 1 {
            arr[(last - out_sample)..last]
                .rchunks_mut(channel as usize)
                .enumerate()
                .for_each(|(i, o_per_channel)| {
                    let gain = i as f64 / out_sample as f64;
                    o_per_channel.iter_mut().for_each(|o| {
                        *o = ((*o as f64) * gain) as f32;
                    });
                });
        }
    }
    fn validate_answer_array(computed: &[f32], answer: &[f32]) {
        computed
            .iter()
            .zip(answer.iter())
            .enumerate()
            .for_each(|(i, (computed, answer))| {
                debug_assert!(
                    (answer - computed).abs() <= f32::EPSILON,
                    "\nindex:{} \ncomputed: {}\nanswer: {}",
                    i,
                    computed,
                    answer
                )
            })
    }
    #[test]
    pub fn run_generator_region() {
        let channel = 2;
        let sample_rate = 48000;
        let range = 0.1..0.2;
        let osc_param = data::generator::OscillatorParam::default();
        let data = data::Region::new(
            AtomicRange::<f64>::new(range.start, range.end),
            data::Content::Generator(data::generator::Generator::Oscillator(
                data::generator::OscillatorFun::SineWave,
                Arc::new(osc_param.clone()),
            )),
            "test_sin",
        );
        let mut model = Model::new(data, channel);
        model.render_offline(sample_rate, channel);
        let range_samps =
            ((range.end - range.start) * sample_rate as f64) as usize * channel as usize;
        assert_eq!(model.interleaved_samples_cache.len(), range_samps);

        let mut answer = vec![0.0f32; range_samps];
        let mut phase = osc_param.phase.get();
        assert_eq!(phase.sin(), 0.0);
        gen_sinewave(
            answer.as_mut_slice(),
            &osc_param,
            &mut phase,
            sample_rate,
            channel as u32,
        );
        assert!(model.cache_completed);
        validate_answer_array(&model.interleaved_samples_cache, &answer);
    }

    fn run_fade_region(in_time: f32, out_time: f32) {
        let fade_param = Arc::new(data::region::FadeParam {
            time_in: in_time.into(),
            time_out: out_time.into(),
        });
        let channel = 2;
        let sample_rate = 48000;
        let range = 0.1..0.2;

        let generator = data::Content::Generator(data::Generator::Constant);
        let range_atomic = AtomicRange::<f64>::new(range.start, range.end);

        let data = data::Region::new(
            range_atomic.clone(),
            data::Content::Transformer(
                data::RegionFilter::FadeInOut(fade_param.clone()),
                Box::new(data::Region::new(range_atomic, generator, "generator")),
            ),
            "test_sin",
        );
        let mut model = Model::new(data, channel);
        model.render_offline(sample_rate, channel);
        let range_samps =
            ((range.end - range.start) * sample_rate as f64) as usize * channel as usize;
        assert_eq!(model.interleaved_samples_cache.len(), range_samps);

        let mut answer = vec![0.0f32; range_samps];

        gen_constant(answer.as_mut_slice(), channel as u32);
        apply_fadeinout(
            answer.as_mut_slice(),
            fade_param.time_in.load().into(),
            fade_param.time_out.load().into(),
            sample_rate,
            channel as u32,
        );
        assert!(model.cache_completed);
        validate_answer_array(&model.interleaved_samples_cache, &answer);
    }
    #[test]
    fn run_fade_zeros() {
        run_fade_region(0.0, 0.0);
    }

    #[test]
    fn run_fade_normal() {
        run_fade_region(0.01, 0.08);
    }
    #[test]
    #[should_panic(expected = "fadein time + fadeout time should be less than region length")]
    fn run_fade_invalid() {
        // fade out is too longer
        run_fade_region(0.05, 0.2);
    }
}
