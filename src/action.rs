//! The definitions of un/redo-able actions for applications.
//! Todo: fix an inconsistensy after code-app translation because serializing/deserializing refreshes Arc references.

use crate::parameter::{FloatParameter, Parameter, RangedNumeric};
use crate::script::{Expr, Value};
use crate::{data, script::param_float};
use std::sync::{Arc, MutexGuard, PoisonError};
use undo;

#[derive(Debug)]
pub enum Error {
    FailToLock(String),
    /// (length of the container, actual index)
    InvalidIndex(usize, i64),
    ContainerEmpty,
    InvalidTrackType,
    NothingToBeAdded, // _Never(PhantomData<T>),
    InvalidConversion,
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
            Self::InvalidConversion => write!(f, "Failed to convert"),
        }
    }
}

impl<T> From<FailedToLockError<'_, Vec<T>>> for Error {
    fn from(e: FailedToLockError<'_, Vec<T>>) -> Self {
        Self::FailToLock(e.to_string())
    }
}

trait DisplayableAction:
    undo::Action<Target = Expr, Output = (), Error = Error> + std::fmt::Display + std::fmt::Debug
{
}

pub struct Action(Box<dyn DisplayableAction<Target = Expr, Output = (), Error = Error>>);

impl<T> From<T> for Action
where
    T: DisplayableAction<Target = Expr, Output = (), Error = Error> + 'static,
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
    type Target = Expr;
    type Output = ();
    type Error = Error;
    fn apply(&mut self, target: &mut Expr) -> undo::Result<Self> {
        self.0.apply(target)
    }
    fn undo(&mut self, target: &mut Expr) -> undo::Result<Self> {
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
    elem: Expr,
    track_num: usize,
    pos: usize,
}
impl AddRegion {
    pub fn new(elem: Expr, track_num: usize) -> Self {
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
    type Target = Expr;

    type Output = ();

    type Error = Error;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match target {
            Expr::Literal(Value::Project(_env, _sr, tracks)) => {
                match tracks.get_mut(self.track_num).unwrap() {
                    Expr::Track(box Expr::Array(regions)) => {
                        regions.push(self.elem.clone());
                        assert!(!regions.is_empty());
                        self.pos = regions.len() - 1;
                        Ok(())
                    }
                    _ => Err(Error::InvalidTrackType),
                }
            }
            _ => Err(Error::InvalidConversion),
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match target {
            Expr::Literal(Value::Project(_env, _sr, tracks)) => {
                match tracks.get_mut(self.track_num).unwrap() {
                    Expr::Track(box Expr::Array(regions)) => {
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
            _ => Err(Error::InvalidConversion),
        }
    }
}

impl DisplayableAction for AddRegion {}

#[derive(Debug)]
pub struct AddTrack {
    elem: Expr,
    pos: usize,
}

impl AddTrack {
    pub fn new(elem: Expr) -> Self {
        Self { elem, pos: 0 }
    }
}
impl undo::Action for AddTrack {
    type Target = Expr;

    type Output = ();

    type Error = Error;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match target {
            Expr::Literal(Value::Project(_env, _sr, tracks)) => {
                tracks.push(self.elem.clone());
                self.pos = tracks.len() - 1;
                Ok(())
            }
            _ => Err(Error::InvalidConversion),
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match target {
            Expr::Literal(Value::Project(_env, _sr, tracks)) => {
                if tracks.is_empty() {
                    Err(Error::ContainerEmpty)
                } else {
                    tracks.remove(self.pos);
                    Ok(())
                }
            }
            _ => Err(Error::InvalidConversion),
        }
    }
}
impl std::fmt::Display for AddTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add track {}", self.pos)
    }
}
impl DisplayableAction for AddTrack {}

#[derive(Debug)]
pub struct AddFadeInOut {
    // elem: Expr, //original region
    pub track_num: usize,
    pub pos: usize,
    pub time_in: f64,
    pub time_out: f64,
}
impl undo::Action for AddFadeInOut {
    type Target = Expr; //project

    type Output = ();

    type Error = Error;

    fn apply(&mut self, project: &mut Self::Target) -> undo::Result<Self> {
        match project {
            Expr::Literal(Value::Project(_env, _sr, tracks)) => {
                match tracks.get_mut(self.track_num).unwrap() {
                    Expr::Track(box Expr::Array(regions)) => {
                        if regions.is_empty() {
                            Err(Error::ContainerEmpty)
                        } else if regions.len() < self.pos {
                            Err(Error::InvalidIndex(regions.len(), self.pos as i64))
                        } else {
                            let reg = regions.get_mut(self.pos).unwrap();
                            let time_in = Expr::Literal(Value::Parameter(Arc::new(param_float!(
                                self.time_in as f32,
                                "time_in",
                                0.0..=10.0
                            ))));
                            let time_out = Expr::Literal(Value::Parameter(Arc::new(param_float!(
                                self.time_out as f32,
                                "time_out",
                                0.0..=10.0
                            ))));
                            *reg = Expr::App(
                                Expr::Var("fadeinout".to_string()).into(),
                                vec![reg.clone(), time_in.into(), time_out.into()],
                            );
                            Ok(())
                        }
                    }
                    _ => Err(Error::InvalidTrackType),
                }
            }
            _ => Err(Error::InvalidConversion),
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match target {
            Expr::Literal(Value::Project(env, _sr, tracks)) => {
                match tracks.get_mut(self.track_num).unwrap() {
                    Expr::Track(box Expr::Array(regions)) => {
                        if regions.is_empty() {
                            Err(Error::ContainerEmpty)
                        } else if regions.len() < self.pos {
                            Err(Error::InvalidIndex(regions.len(), self.pos as i64))
                        } else {
                            let reg = regions.get_mut(self.pos).unwrap();
                            if let Expr::App(box Expr::Var(name), r) = reg {
                                match (name.as_str(), r.as_slice()) {
                                    ("fadeinout", [v, time_in, time_out]) => {
                                        self.time_in = time_in
                                            .eval(env.clone(), &None)
                                            .and_then(|v| v.get_as_float())
                                            .unwrap_or(0.0);
                                        self.time_out = time_out
                                            .eval(env.clone(), &None)
                                            .and_then(|v| v.get_as_float())
                                            .unwrap_or(0.0);

                                        *reg = v.clone()
                                    }
                                    _ => return Err(Error::InvalidConversion),
                                }
                            }
                            Ok(())
                        }
                    }
                    _ => Err(Error::InvalidTrackType),
                }
            }
            _ => Err(Error::InvalidConversion),
        }
    }
}
impl std::fmt::Display for AddFadeInOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add fadein/out to region")
    }
}
impl DisplayableAction for AddFadeInOut {}
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
