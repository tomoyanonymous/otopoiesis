use crate::audio::{get_component_for_value, PlaybackInfo, RangedComponent, RangedComponentDyn};

// use crate::parameter::UIntParameter
use crate::data::{self, Region};
use crate::parameter::Parameter;
use crate::script::{Expr, Value};
use crate::utils::{AtomicRange, SimpleAtomic};
use std::ops::RangeInclusive;

// 基本はオフラインレンダリング

#[derive(Debug)]
pub struct FadeModel {
    pub param: data::FadeParam,
    pub origin: Box<Model>,
}
impl FadeModel {
    fn new(p: data::FadeParam, origin: data::Region) -> Self {
        Self {
            param: p,
            origin: Box::new(Model::new(origin, 2)),
        }
    }
}

impl RangedComponent for FadeModel {
    fn get_range(&self) -> RangeInclusive<f64> {
        self.origin.params.getrange()
    }
    fn get_output_channels(&self) -> u64 {
        2
    }

    fn render_offline(&mut self, sample_rate: u32, channels: u64) {
        // resize should be the caller.
        // dest.resize(self.origin.interleaved_samples_cache.len(), 0.0);
        todo!();
        self.origin.render_offline(sample_rate, channels);
        let dest = self.get_sample_cache_mut();
        assert_eq!(self.origin.interleaved_samples_cache.len(), dest.len());
        let chs = self.get_output_channels() as usize;
        let in_time = (self.param.time_in.get() as f64 * sample_rate as f64) as usize;
        let out_time = (self.param.time_out.get() as f64 * sample_rate as f64) as usize;
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

    fn get_sample_cache(&self) -> &[f32] {
        todo!()
    }

    fn get_sample(&self, time: f64, sample_rate: u32) -> Option<f64> {
        todo!()
    }

    fn get_sample_cache_mut(&mut self) -> &mut [f32] {
        todo!()
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
        todo!()
        // if !self.0.is_empty() {
        //     let start = self.0[0].params.range.start();
        //     let end = self.0.last().unwrap().params.range.end();
        //     assert!(end >= start);
        //     start..=end
        // } else {
        //     0.0..=0.0
        // }
    }

    fn get_output_channels(&self) -> u64 {
        2
    }

    fn render_offline(&mut self, sample_rate: u32, channels: u64) {
        //todo: asynchrounous render
        todo!();
        // self.0.iter_mut().for_each(|region| {
        //     let range = &region.params.range;
        //     let scale_to_index = |x: f64| (x * sample_rate as f64) as usize * channels as usize;
        //     region
        //         .interleaved_samples_cache
        //         .resize(scale_to_index(range.getrange()), 0.0); // no need?
        //     region.render_offline(sample_rate, channels);
        //     self.get_sample_cache_mut()
        //         .copy_from_slice(&region.interleaved_samples_cache);
        // });
    }

    fn get_sample(&self, time: f64, sample_rate: u32) -> Option<f64> {
        todo!()
    }

    fn get_sample_cache(&self) -> &[f32] {
        todo!()
    }

    fn get_sample_cache_mut(&mut self) -> &mut [f32] {
        todo!()
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
            data::RegionFilter::Script(val) => {
                match val {
                    Value::Closure(_ids, env, box Expr::App(box Expr::Var(fname), args)) => {
                        todo!()
                        // match (fname.as_str(), args.as_slice()) {
                        //     (
                        //         "apply_fade_in_out",
                        //         [Expr::Literal(region), Expr::Literal(Value::Parameter(time_in)), Expr::Literal(Value::Parameter(time_out))],
                        //     ) => {
                        //         let (start, dur) =
                        //             if let Value::Region(start, dur, _content, _label, _t) = region
                        //             {
                        //                 (start, dur)
                        //             } else {
                        //                 panic!("not a region")
                        //             };
                        //         Box::new(RangedScriptComponent::new(
                        //             Value::Parameter(start.clone()),
                        //             Value::Parameter(dur.clone()),
                        //             region.clone(),
                        //             Value::ExtFunction(()),
                        //             env.clone(),
                        //         ))
                        //     }
                        //     _ => todo!(),
                        // }
                    }
                    _ => todo!(),
                }
                //rg and origin should be same...
            }
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

        let c = get_component_for_value(&params.content);
        let content = Box::new(RangedComponentDyn::new(
            c,
            AtomicRange::new(params.start.clone(), params.dur.clone()),
        ));

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
            (self.params.dur.get() as f64 * sample_rate as f64) as usize * channels as usize,
            0.0,
        );
        self.content.render_offline(sample_rate, channels);
        self.interleaved_samples_cache = self.content.get_sample_cache().to_vec();
        self.cache_completed = true;
    }
    pub fn contains_samples(&self, range: RangeInclusive<f64>) -> bool {
        let t_range = &self.params.getrange();
        let start = t_range.start();
        let end = t_range.end();
        let c1 = range.contains(&start) || range.contains(&end);
        let c2 = t_range.contains(range.start()) && t_range.contains(range.end());
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
            r.render_offline(info.sample_rate, info.channels);
            r
        })
        .expect("failed to launch thread")
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{
        data::Content,
        parameter::{FloatParameter, Parameter, RangedNumeric},
        script::{builtin_fn, Environment, Expr, ExtFun, Value},
    };

    use super::*;

    fn gen_sinewave(
        arr: &mut [f32],
        osc_param: &data::OscillatorParam,
        phase_init: f32,
        sample_rate: u32,
        channel: u32,
    ) {
        let mut phase = phase_init;
        let twopi = std::f32::consts::PI * 2.0;
        arr.chunks_mut(channel as usize)
            .enumerate()
            .for_each(|(_count, o_per_channel)| {
                // let now = count as f64 / sample_rate as f64;
                o_per_channel.iter_mut().enumerate().for_each(|(ch, o)| {
                    if ch == 0 {
                        *o = (phase * twopi).sin() * osc_param.amp.get();
                    } else {
                        *o = 0.0;
                    }
                });
                phase = (phase + osc_param.freq.get() / (sample_rate as f32)) % 1.0;
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
        let start = Arc::new(FloatParameter::new(0.1, "start"));
        let dur = Arc::new(FloatParameter::new(0.1, "dur"));
        let osc_param = data::generator::OscillatorParam::default();
        let data = data::Region::new(
            start.clone(),
            dur.clone(),
            Value::ExtFunction(ExtFun::new(builtin_fn::SineWave::new())),
            "test_sin",
        );
        let mut model = Model::new(data, channel);
        model.render_offline(sample_rate, channel);
        let range_samps = (dur.get() as f64 * sample_rate as f64) as usize * channel as usize;
        assert_eq!(model.interleaved_samples_cache.len(), range_samps);

        let mut answer = vec![0.0f32; range_samps];
        let phase = osc_param.phase.get();
        assert_eq!(phase.sin(), 0.0);
        gen_sinewave(
            answer.as_mut_slice(),
            &osc_param,
            phase,
            sample_rate,
            channel as u32,
        );
        assert!(model.cache_completed);
        validate_answer_array(&model.interleaved_samples_cache, &answer);
    }

    fn run_fade_region(in_time: f32, out_time: f32) {
        let time_in = FloatParameter::new(in_time, "time_in").set_range(0.0..=1000.0);
        let time_out = FloatParameter::new(out_time, "time_out").set_range(0.0..=1000.0);
        let channels = 2;
        let sample_rate = 48000;
        let start = FloatParameter::new(0.1, "start");
        let dur = FloatParameter::new(0.1, "dur");

        let generator = Expr::App(
            Expr::Literal(Value::new_lazy(Expr::Literal(Value::Number(1.0))).into()).into(),
            vec![],
        );

        let region = Expr::Region(
            Expr::Literal(Value::new_param(start.clone())).into(),
            Expr::Literal(Value::new_param(dur.clone())).into(),
            generator.into(),
            "generator".into(),
        );
        let region_with_fade = Expr::App(
            Expr::Var("fadeinout".to_string()).into(),
            vec![
                region.clone(),
                Expr::Literal(Value::new_param(time_in.clone())),
                Expr::Literal(Value::new_param(time_out.clone())),
            ],
        );
        let env = Arc::new(Environment::new());
        let info = PlaybackInfo {
            sample_rate,
            current_time: 0,
            frame_per_buffer: 256,
            channels,
        };
        let region_res = region_with_fade.eval(env, &Some(&info), &mut None).unwrap();
        let data = data::Region::try_from(&region_res).unwrap();
        let mut model = Model::new(data, channels);
        model.render_offline(sample_rate, channels);
        let range_samps = (dur.get() as f64 * sample_rate as f64) as usize * channels as usize;
        assert_eq!(model.interleaved_samples_cache.len(), range_samps);

        let mut answer = vec![1.0f32; range_samps];

        gen_constant(answer.as_mut_slice(), channels as u32);
        apply_fadeinout(
            answer.as_mut_slice(),
            time_in.get().into(),
            time_out.get().into(),
            sample_rate,
            channels as u32,
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
