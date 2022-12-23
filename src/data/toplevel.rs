use crate::parameter::FloatParameter;
/// Data structure which is kept as project file.
/// This data structure can't be shared between threads.
/// This data will be compiled into the sharable data structure to use in gui and audio interpreter.
use serde::{Deserialize, Serialize};
//基本的にリアルタイムでtweakしたいパラメーター以外はArcで包む必要はない。再生直前にコンパイルし直して丸ごとClone⇨Audioスレッドに送信でいける

/// A main project data. It should be imported/exported via serde.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub sample_rate: u64,
    pub tracks: Vec<Track>,
}

/// Data structure for track.
/// The track has some input/output stream.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Track {
    ///Contains Multiple Regions.
    Regions(Vec<Region>),
    ///Contains one audio generator(0 input).
    Generator(Generator),
    ///TODO: Take another track and transform it (like filter).
    Transformer(),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OscillatorParam {
    pub amp: FloatParameter,
    pub freq: FloatParameter,
    pub phase: FloatParameter,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OscillatorFun {
    SineWave,
    /// up or down
    SawTooth(bool),
    // Duty Ratio
    Rectanglular(f32),
    Triangular,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Generator {
    Oscillator(OscillatorFun, OscillatorParam),
    Noise(),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Region {
    /// range stores a real time, not in sample.
    pub range: std::ops::RangeInclusive<f32>,
    pub content: Content,
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RegionFilter {
    Gain,
    FadeInOut(f32, f32),
    Reverse,
    Replicate(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Content {
    Generator(Generator),
    AudioFile(),
    // region transformer is an recursive data
    Transformer(RegionFilter, Box<Region>),
}

#[cfg(test)]
mod test {
    use super::*;
    trait Sendable: Send {}
    // ensure that Project can be sent to audio thread.
    impl Sendable for Project {}
}
