use std::string;
use std::sync::Arc;

/// Generator is a similar concept to Unit Generator in the other popular sound programming environments.
/// These generators are loaded from Region or Track.
///
use crate::data::atomic;
use crate::parameter::{FloatParameter, UIntParameter, Parameter};
use serde::{Deserialize, Serialize};
/// Utility Parameter for oscillator with some default values.

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OscillatorParam {
    pub amp: FloatParameter,
    pub freq: FloatParameter,
    pub phase: FloatParameter,
}
impl Default for OscillatorParam {
    fn default() -> Self {
        Self {
            amp: FloatParameter::new(0.8, 0.0..=1.0, "amp"),
            freq: FloatParameter::new(440.0, 20.0..=20000.0, "freq"),
            phase: FloatParameter::new(0.0, 0.0..=std::f32::consts::PI * 2.0, "phase"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OscillatorFun {
    SineWave,
    /// up or down
    SawTooth(Arc<atomic::Bool>),
    // Duty Ratio
    Rectanglular(Arc<atomic::F32>),
    Triangular,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilePlayerParam{
    pub path: String,
    pub channels: UIntParameter,
    pub start_sec: FloatParameter,
    pub duration: FloatParameter
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Generator {
    Oscillator(OscillatorFun, Arc<OscillatorParam>),
    Noise(),
    ///mostly for debugging filter.
    Constant,
}

impl std::default::Default for Generator {
    fn default() -> Self {
        Self::Oscillator(
            OscillatorFun::SineWave,
            Arc::new(OscillatorParam::default()),
        )
    }
}
