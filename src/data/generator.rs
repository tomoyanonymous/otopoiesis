use std::sync::Arc;

/// Generator is a similar concept to Unit Generator in the other popular sound programming environments.
/// These generators are loaded from Region or Track.
///
use crate::data::atomic;
use crate::parameter::{FloatParameter, Parameter, UIntParameter, RangedNumeric};
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
            amp: FloatParameter::new(1.0, "amp").set_range( 0.0..=1.0),
            freq: FloatParameter::new(440.0, "freq").set_range( 0.01..=20000.0),
            phase: FloatParameter::new(0.0, "phase").set_range( 0.0..=std::f32::consts::PI * 2.0),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum OscillatorFun {
    SineWave,
    /// up or down
    SawTooth(Arc<atomic::Bool>),
    // Duty Ratio
    Rectanglular(Arc<atomic::F32>),
    Triangular,
}
impl Default for OscillatorFun {
    fn default() -> Self {
        OscillatorFun::SineWave
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilePlayerParam {
    pub path: String,
    pub channels: UIntParameter,
    pub start_sec: FloatParameter,
    pub duration: FloatParameter,
}

impl FilePlayerParam {
    pub fn new_test_file() -> (Self, usize) {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test/assets/test-voice-stereo.wav"
        )
        .to_string();
        let length_in_samples = 119608;
        (
            Self {
                path,
                channels: UIntParameter::new(2, "channels").set_range(0..=2),
                start_sec: FloatParameter::new(0.0, "start").set_range( 0.0..=10.0),
                duration: FloatParameter::new(1.0, "duration").set_range( 0.0..=10.0),
            },
            length_in_samples,
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Generator {
    Oscillator(OscillatorFun, Arc<OscillatorParam>),
    Noise(),
    ///mostly for debugging filter.
    Constant(Arc<FloatParameter>),
    #[cfg(not(feature = "web"))]
    FilePlayer(Arc<FilePlayerParam>),
}

impl std::default::Default for Generator {
    fn default() -> Self {
        Self::Oscillator(
            OscillatorFun::SineWave,
            Arc::new(OscillatorParam::default()),
        )
    }
}
