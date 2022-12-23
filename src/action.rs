//! The definitions of un/redo-able actions for applications.
//! Todo: fix an inconsistensy after code-app translation because serializing/deserializing refreshes Arc references.

use crate::data::{self};
use std::sync::{ MutexGuard, PoisonError};
use undo;

#[derive(Debug)]
pub enum OpsContainerError {
    FailToLock(String),
    ContainerEmpty,
    NothingToBeAdded, // _Never(PhantomData<T>),
}
pub type FailedToLockError<'a, T> = PoisonError<MutexGuard<'a, T>>;

impl std::fmt::Display for OpsContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContainerEmpty => write!(f, "tried to remove container that was empty"),
            Self::NothingToBeAdded => write!(f, "Nothing to be Added"),
            Self::FailToLock(msg) => write!(f, "{msg}"),
        }
    }
}

impl<T> From<FailedToLockError<'_, Vec<T>>> for OpsContainerError {
    fn from(e: FailedToLockError<'_, Vec<T>>) -> Self {
        Self::FailToLock(e.to_string())
    }
}

trait DisplayableAction: undo::Action + std::fmt::Display {}

pub struct Action(Box<dyn DisplayableAction<Target = data::Project, Output = (), Error = ()>>);

impl<T> From<T> for Action
where
    T: DisplayableAction<Target = data::Project, Output = (), Error = ()> + 'static,
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
    type Error = ();
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
//     a: impl DisplayableAction<Target = (), Output = (), Error = OpsContainerError> + 'static,
// ) -> Action {
//     Action(Box::new(a))
// }

#[derive(Debug)]
struct AddRegion {
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
        write!(f, "add region")
    }
}

impl undo::Action for AddRegion {
    type Target = data::Project;

    type Output = ();

    type Error = ();

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match target.tracks.get_mut(self.track_num).unwrap() {
            data::Track::Regions(regions) => {
                regions.push(self.elem.clone());
                self.pos = regions.len();
            }
            data::Track::Generator(_) => todo!(),
            data::Track::Transformer() => todo!(),
        }
        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match target.tracks.get_mut(self.track_num).unwrap() {
            data::Track::Regions(regions) => {
                regions.remove(self.pos);
            }
            data::Track::Generator(_) => todo!(),
            data::Track::Transformer() => todo!(),
        }
        Ok(())
    }
}

impl DisplayableAction for AddRegion {}
struct AddTrack {
    elem: data::Track,
    pos: usize,
}

impl AddTrack {
    fn new(elem: data::Track) -> Self {
        Self { elem, pos: 0 }
    }
}
impl undo::Action for AddTrack {
    type Target = data::Project;

    type Output = ();

    type Error = ();

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        target.tracks.push(self.elem.clone());
        self.pos = target.tracks.len();
        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        target.tracks.remove(self.pos);
        Ok(())
    }
}
impl std::fmt::Display for AddTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add track")
    }
}
impl DisplayableAction for AddTrack {}

pub fn add_region(
    app: &mut data::AppModel,
    track_num: usize,
    region: data::Region,
) -> Result<(), ()> {
    app.history.apply(
        &mut app.project,
        Action::from(AddRegion::new(region, track_num)),
    )
}
pub fn add_track(app: &mut data::AppModel, track: data::Track) -> Result<(), ()> {
    app.history
        .apply(&mut app.project, Action::from(AddTrack::new(track)))
}
