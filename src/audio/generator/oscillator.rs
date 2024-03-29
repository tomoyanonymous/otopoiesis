use super::GeneratorComponent;
use crate::audio::PlaybackInfo;
use crate::data::OscillatorParam;
use crate::parameter::FloatParameter;
use crate::{data, parameter::Parameter};
use std::f32::consts::PI;
use std::sync::Arc;
const TWOPI: f32 = PI * 2.0;

pub trait Oscillator {
    fn get_params(&self) -> &OscillatorParam;

    fn set_phase(&mut self, init: f32);
    fn phase(&self) -> f32;

    fn map(&self, phase: f32) -> f32;
}
impl<T: Oscillator> GeneratorComponent for T {
    type Params = OscillatorParam;

    fn get_params(&self) -> &Self::Params {
        self.get_params()
    }

    fn reset_phase(&mut self) {
        self.set_phase(self.get_params().phase.get())
    }

    fn render_sample(&mut self, out: &mut f32, info: &PlaybackInfo) {
        *out = self.map(self.phase()) * self.get_params().amp.get();
        self.set_phase(
            (self.phase() + self.get_params().freq.get() / info.sample_rate as f32) % 1.0,
        );
    }
}

pub struct GenericOscillator {
    pub params: data::OscillatorParam,
    phase_internal: f32,
    map_fn: Arc<dyn Fn(f32) -> f32 + 'static + Send + Sync>,
}
impl Clone for GenericOscillator {
    fn clone(&self) -> Self {
        Self {
            params: self.params.clone(),
            phase_internal: self.phase_internal,
            map_fn: Arc::clone(&self.map_fn),
        }
    }
}

impl std::fmt::Display for GenericOscillator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?},{:?}", self.params, self.phase_internal)
    }
}
impl std::fmt::Debug for GenericOscillator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl GenericOscillator {
    pub fn new<F>(params: data::OscillatorParam, map_fn: F) -> Self
    where
        F: Fn(f32) -> f32 + 'static + Send + Sync,
    {
        Self {
            params: params.clone(),
            phase_internal: params.phase.get(),
            map_fn: Arc::new(map_fn),
        }
    }
}

impl Oscillator for GenericOscillator {
    fn get_params(&self) -> &OscillatorParam {
        &self.params
    }

    fn set_phase(&mut self, init: f32) {
        self.phase_internal = init;
    }

    fn phase(&self) -> f32 {
        self.phase_internal
    }

    fn map(&self, phase: f32) -> f32 {
        (self.map_fn)(phase)
    }
}

pub fn sinewave(params: data::OscillatorParam) -> GenericOscillator {
    GenericOscillator::new(params, move |phase: f32| (phase * TWOPI).sin())
}
pub fn saw(params: data::OscillatorParam, direction: Arc<FloatParameter>) -> GenericOscillator {
    GenericOscillator::new(params, move |phase: f32| {
        let dir = direction.get() > 1.0;
        (phase * 2.0 - 1.0) * if dir { 1.0 } else { -1.0 }
    })
}
pub fn rect(params: data::OscillatorParam, duty: Arc<FloatParameter>) -> GenericOscillator {
    GenericOscillator::new(
        params,
        move |phase: f32| {
            if phase > duty.get() {
                1.0
            } else {
                -1.0
            }
        },
    )
}
pub fn triangle(params: data::OscillatorParam) -> GenericOscillator {
    GenericOscillator::new(params, move |phase: f32| {
        (phase * 2.0 - 1.0).abs() * 2.0 - 1.0
    })
}
