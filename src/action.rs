//! The definitions of un/redo-able actions for applications.
//! Todo: fix an inconsistensy after code-app translation because serializing/deserializing refreshes Arc references.

use crate::data::{self};
use std::sync::{MutexGuard, PoisonError};
use undo;

#[derive(Debug)]
pub enum Error {
    FailToLock(String),
    /// (length of the container, actual index)
    InvalidIndex(usize, i64),
    ContainerEmpty,
    InvalidTrackType,
    NothingToBeAdded, // _Never(PhantomData<T>),
}
pub type FailedToLockError<'a, T> = PoisonError<MutexGuard<'a, T>>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContainerEmpty => write!(f, "tried to remove container that was empty"),
            Self::InvalidIndex(len, index) => write!(
                f,
                "Invalid Index to the Container,length was {} but the index was {}",
                len, index
            ),
            Self::NothingToBeAdded => write!(f, "Nothing to be Added"),
            Self::FailToLock(msg) => write!(f, "{msg}"),
            Self::InvalidTrackType => write!(f, "Track type was not an array of regions"),
        }
    }
}

impl<T> From<FailedToLockError<'_, Vec<T>>> for Error {
    fn from(e: FailedToLockError<'_, Vec<T>>) -> Self {
        Self::FailToLock(e.to_string())
    }
}

trait DisplayableAction:
    undo::Action<Target = data::Project, Output = (), Error = Error>
    + std::fmt::Display
    + std::fmt::Debug
{
}

pub struct Action(Box<dyn DisplayableAction<Target = data::Project, Output = (), Error = Error>>);

impl<T> From<T> for Action
where
    T: DisplayableAction<Target = data::Project, Output = (), Error = Error> + 'static,
{
    fn from(v: T) -> Self {
        Self(Box::new(v))
    }
}

pub enum Target {
    Tracks(data::Project),
    Regions(Vec<data::Region>),
}

impl undo::Action for Action {
    type Target = data::Project;
    type Output = ();
    type Error = Error;
    fn apply(&mut self, target: &mut data::Project) -> undo::Result<Self> {
        self.0.apply(target)
    }
    fn undo(&mut self, target: &mut data::Project) -> undo::Result<Self> {
        self.0.undo(target)
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// fn make_action_dyn(
//     a: impl DisplayableAction<Target = (), Output = (), Error = Error> + 'static,
// ) -> Action {
//     Action(Box::new(a))
// }

#[derive(Debug)]
pub struct AddRegion {
    elem: data::Region,
    track_num: usize,
    pos: usize,
}
impl AddRegion {
    pub fn new(elem: data::Region, track_num: usize) -> Self {
        Self {
            elem,
            track_num,
            pos: 0,
        }
    }
}

impl std::fmt::Display for AddRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add Region {} in Track {}", self.pos, self.track_num)
    }
}

impl undo::Action for AddRegion {
    type Target = data::Project;

    type Output = ();

    type Error = Error;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match target.tracks.get_mut(self.track_num).unwrap() {
            data::Track::Regions(regions) => {
                regions.push(self.elem.clone());
                assert!(!regions.is_empty());
                self.pos = regions.len() - 1;
                Ok(())
            }
            _ => Err(Error::InvalidTrackType),
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match target.tracks.get_mut(self.track_num).unwrap() {
            data::Track::Regions(regions) => {
                if regions.is_empty() {
                    Err(Error::ContainerEmpty)
                } else if regions.len() < self.pos {
                    Err(Error::InvalidIndex(regions.len(), self.pos as i64))
                } else {
                    regions.remove(self.pos);
                    Ok(())
                }
            }
            _ => Err(Error::InvalidTrackType),
        }
    }
}

impl DisplayableAction for AddRegion {}

#[derive(Debug)]
pub struct AddTrack {
    elem: data::Track,
    pos: usize,
}

impl AddTrack {
    pub fn new(elem: data::Track) -> Self {
        Self { elem, pos: 0 }
    }
}
impl undo::Action for AddTrack {
    type Target = data::Project;

    type Output = ();

    type Error = Error;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        target.tracks.push(self.elem.clone());
        self.pos = target.tracks.len() - 1;
        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        if target.tracks.is_empty() {
            Err(Error::ContainerEmpty)
        } else {
            target.tracks.remove(self.pos);
            Ok(())
        }
    }
}
impl std::fmt::Display for AddTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add track {}", self.pos)
    }
}
impl DisplayableAction for AddTrack {}

// pub fn add_region(
//     app: &mut data::AppModel,
//     track_num: usize,
//     region: data::Region,
// ) -> Result<(), Error> {
//     app.history.apply(
//         &mut app.project,
//         Action::from(AddRegion::new(region, track_num)),
//     )
// }
// pub fn add_track(app: &mut data::AppModel, track: data::Track) -> Result<(), Error> {
//     app.history
//         .apply(&mut app.project, Action::from(AddTrack::new(track)))
// }
