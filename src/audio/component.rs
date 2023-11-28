use std::{ops::RangeInclusive, sync::Arc};

use crate::{
    data::{ConversionError, Region},
    script::{self, Environment, EvalError, Expr, Value},
    utils::AtomicRange,
};

use super::{generator::fileplayer::FilePlayer, region, PlaybackInfo};
pub trait Component: std::fmt::Debug {
    fn get_input_channels(&self) -> u64;
    fn get_output_channels(&self) -> u64;
    fn prepare_play(&mut self, info: &PlaybackInfo);
    fn render(&mut self, input: &[f32], output: &mut [f32], info: &PlaybackInfo);
}

#[derive(Debug)]
pub struct ScriptComponent {
    val: Value,
}

impl ScriptComponent {
    pub fn try_new(val: &Value) -> Result<Self, EvalError> {
        let res = match val {
            Value::Closure(_ids, _env, box _body) => true,
            _ => false,
        };
        if res {
            Ok(Self { val: val.clone() })
        } else {
            Err(EvalError::TypeMismatch("not a closure".into()))
        }
    }
    fn compute_sample(&self, info: &PlaybackInfo) -> f64 {
        match &self.val {
            Value::Closure(_ids, env, box body) => {
                match body.eval(env.clone(), &Some(info), &mut None) {
                    Ok(Value::Number(res)) => res,
                    _ => 0.0,
                }
            }
            _ => 0.0,
        }
    }
}
impl TryFrom<&Value> for ScriptComponent {
    type Error = ConversionError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Self::try_new(value).map_err(|_| ConversionError {})
    }
}
impl Component for ScriptComponent {
    fn get_input_channels(&self) -> u64 {
        if let Value::Closure(ids, _env, box _body) = &self.val {
            ids.len() as u64
        } else {
            0
        }
    }

    fn get_output_channels(&self) -> u64 {
        //todo!
        2
    }

    fn prepare_play(&mut self, _info: &PlaybackInfo) {
        //do nothing
    }

    fn render(&mut self, _input: &[f32], output: &mut [f32], info: &PlaybackInfo) {
        let mut info = info.clone(); //todo: inefficient
        for (_count, out_per_channel) in output
            .chunks_mut(self.get_output_channels() as usize)
            .enumerate()
        {
            info.current_time += 1;
            for (ch, s) in out_per_channel.iter_mut().enumerate() {
                if ch == 0 {
                    *s = self.compute_sample(&info) as f32;
                } else {
                    *s = 0.0
                }
            }
        }
    }
}

pub fn get_component_for_value(v: &script::Value) -> Box<dyn Component + Send + Sync> {
    FilePlayer::try_from(v)
        .map(|c| Box::new(c) as Box<dyn Component + Send + Sync>)
        .or(ScriptComponent::try_from(v).map(|c| Box::new(c) as Box<dyn Component + Send + Sync>))
        .expect("not a valid component")
}

/// Interface for offline rendering.
pub trait RangedComponent: std::fmt::Debug {
    fn get_range(&self) -> RangeInclusive<f64>;
    fn get_output_channels(&self) -> u64;
    fn get_sample_cache(&self) -> &[f32];
    fn get_sample_cache_mut(&mut self) -> &mut [f32];
    fn render_offline(&mut self, sample_rate: u32, channels: u64);
    fn get_cache_len(&self, sample_rate: u32) -> usize {
        let range = self.get_range();
        let len_sec = range.end() - range.start();
        (len_sec * sample_rate as f64) as usize
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
    fn get_default_playback_info(&self, sample_rate: u32, channels: u64) -> PlaybackInfo {
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
            region.render_offline(info.sample_rate, info.channels);
            region
        })
        .expect("failed to launch thread")
}

//convert any generator component into region

#[derive(Debug)]
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
    pub fn from_value(value: &script::Value, range: AtomicRange) -> Self {
        let c = get_component_for_value(value);
        Self::new(c, range)
    }
}

impl RangedComponent for GenericRangedComponent {
    fn get_range(&self) -> RangeInclusive<f64> {
        self.range.start() as f64..=self.range.end() as f64
    }

    fn get_output_channels(&self) -> u64 {
        self.generator.get_output_channels()
    }

    fn render_offline(&mut self, sample_rate: u32, channels: u64) {
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

#[derive(Debug)]
pub struct RangedScriptComponent {
    pub start: Value,
    pub dur: Value,
    pub origin: Value, //expect:region
    pub translator: Value,
    pub env: Arc<Environment>,
    cache: Vec<f32>,
}
impl RangedScriptComponent {
    pub fn new(
        start: Value,
        dur: Value,
        origin: Value,
        translator: Value,
        env: Arc<Environment>,
    ) -> Self {
        Self {
            start,
            dur,
            origin,
            translator,
            env,
            cache: vec![],
        }
    }
    pub fn compute_sample(&self, input: f64, info: &PlaybackInfo) -> f64 {
        let expr = Expr::App(
            //例えばFadeInOutなら、クロージャにtime_in、time_outの様なパラメータを閉じ込めておいて、apply_fade_in_outの中でenv.lookupで取り出す、とか？
            Expr::Literal(self.translator.clone()).into(),
            vec![
                Expr::Literal(Value::Number(input)),
                Expr::Literal(self.start.clone()),
                Expr::Literal(self.dur.clone()),
            ],
        );
        let sample = expr.eval(self.env.clone(), &Some(info), &mut None);
        sample.unwrap().get_as_float().unwrap()
    }
}

impl RangedComponent for RangedScriptComponent {
    fn get_range(&self) -> RangeInclusive<f64> {
        let start = self.start.get_as_float().unwrap();
        let dur = self.dur.get_as_float().unwrap();
        start..=(start + dur)
    }

    fn get_output_channels(&self) -> u64 {
        2
    }

    fn render_offline(&mut self, sample_rate: u32, channels: u64) {
        //リバースとかノンリニアな処理のものとフェードインアウトとかリニアな処理（リアルタイム加工できるもの）は形を分けるべきかもしれない
        self.cache.resize(self.get_cache_len(sample_rate), 0.0);
        let mut info = self.get_default_playback_info(sample_rate, channels);
        let origin_rg = Region::try_from(&self.origin).expect("not a region");
        let mut origin_model = region::Model::new(origin_rg, 2);
        origin_model.render_offline(sample_rate, channels);
        let mut dest = self.cache.clone();
        for (_count, (input_per_channel, out_per_channel)) in origin_model
            .interleaved_samples_cache
            .chunks(channels as usize)
            .zip(dest.chunks_mut(channels as usize))
            .enumerate()
        {
            info.current_time += 1;
            for (ch, (i, o)) in input_per_channel
                .iter()
                .zip(out_per_channel.iter_mut())
                .enumerate()
            {
                if ch == 0 {
                    *o = self.compute_sample(*i as f64, &info) as f32;
                } else {
                    *o = 0.0
                }
            }
        }
        self.cache = dest;
    }

    fn get_sample_cache(&self) -> &[f32] {
        &self.cache
    }

    fn get_sample_cache_mut(&mut self) -> &mut [f32] {
        &mut self.cache
    }
}
