use serde::{Deserialize,Serialize};
use super::{Region,Generator};

/// Data structure for track.
/// The track has some input/output stream.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Track {
    ///Contains Multiple Regions.
    /// TODO:Change container for this to be HashedSet for the more efficient implmentation of Undo Action.
    Regions(Vec<Region>),
    ///Contains one audio generator(0 input).
    Generator(Generator),
    ///Take another track and transform it (like filter).
    Transformer(),
}

impl Track {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Track {
    fn default() -> Self {
        Track::Regions(vec![])
    }
}
impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "track {}", self.label)
        write!(f, "track")
    }
}
