//! The definitions of un/redo-able actions for applications.
//! Todo: fix an inconsistensy after code-app translation because serializing/deserializing refreshes Arc references.

use crate::data::AppModel;
use crate::parameter::{FloatParameter, Parameter, RangedNumeric};
use crate::script::{Expr, Value};
use crate::{data, script::param_float};
use script::expr::{ExprRef, Literal};
use script::parser::{ParseContext, ParseContextRef};
use script::{value, Symbol};
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
    InvalidAllocation,
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
            Self::InvalidAllocation => write!(f, "Failed to get expr from storage"),
        }
    }
}

impl<T> From<FailedToLockError<'_, Vec<T>>> for Error {
    fn from(e: FailedToLockError<'_, Vec<T>>) -> Self {
        Self::FailToLock(e.to_string())
    }
}
fn get_expr<'a>(ctx: &ParseContext, e: &ExprRef) -> Result<&'a Expr, Error> {
    ctx.get_expr(e.clone()).ok_or(Error::InvalidAllocation)
}
fn get_track_mut<'a>(app: &mut AppModel) -> Result<&'a mut Vec<ExprRef>, Error> {
    let ctx = app.compile_ctx.parsectx;
    if let Some(mut proj) = app.source {
        let e = get_expr(&ctx, &proj)?;
        match e {
            Expr::Project(_sr, mut tracks) => Ok(&mut tracks),
            _ => Err(Error::InvalidConversion),
        }
    } else {
        Err(Error::InvalidConversion)
    }
}
fn get_regions_mut<'a>(
    app: &mut AppModel,
    track_num: usize,
) -> Result<&'a mut Vec<ExprRef>, Error> {
    let ctx = app.compile_ctx.parsectx;

    let tracks = get_track_mut(app)?;
    let ts = tracks
        .get_mut(track_num)
        .ok_or(Error::InvalidIndex(tracks.len(), track_num as i64))
        .and_then(|track| get_expr(&ctx, &track))?;
    let mut regions = match ts {
        Expr::Track(array) => get_expr(&ctx, &array),
        _ => Err(Error::InvalidTrackType),
    }?;
    match regions {
        Expr::Array(mut regions) => Ok(&mut regions),
        _ => Err(Error::InvalidTrackType),
    }
}

fn get_region<'a>(
    app: &mut AppModel,
    track_num: usize,
    region_num: usize,
) -> Result<&mut ExprRef, Error> {
    let ctx = app.compile_ctx.parsectx;
    let regions = get_regions_mut(app, track_num)?;
    regions.get_mut(region_num).ok_or(Error::InvalidAllocation)
}

#[derive(Debug)]
pub struct AddRegion {
    elem: ExprRef,
    track_num: usize,
    pos: usize,
}
impl AddRegion {
    pub fn new(elem: ExprRef, track_num: usize) -> Self {
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
    type Target = AppModel;
    type Output = ();
    type Error = Error;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let mut regions = get_regions_mut(target, self.track_num)?;
        regions.push(self.elem.clone());
        self.pos = regions.len() - 1;
        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let mut regions = get_regions_mut(target, self.track_num)?;
        if regions.is_empty() {
            Err(Error::ContainerEmpty)
        } else if regions.len() < self.pos {
            Err(Error::InvalidIndex(regions.len(), self.pos as i64))
        } else {
            regions.remove(self.pos);
            Ok(())
        }
    }
}

// impl DisplayableAction for AddRegion {}

#[derive(Debug)]
pub struct AddTrack {
    elem: ExprRef,
    pos: usize,
}

impl AddTrack {
    pub fn new(elem: ExprRef) -> Self {
        Self { elem, pos: 0 }
    }
}
impl undo::Action for AddTrack {
    type Target = AppModel;

    type Output = ();

    type Error = Error;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let tracks = get_track_mut(target)?;
        tracks.push(self.elem.clone());
        self.pos = tracks.len() - 1;
        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let tracks = get_track_mut(target)?;
        if tracks.is_empty() {
            Err(Error::ContainerEmpty)
        } else {
            tracks.remove(self.pos);
            Ok(())
        }
    }
}
impl std::fmt::Display for AddTrack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add track {}", self.pos)
    }
}
// impl DisplayableAction for AddTrack {}

#[derive(Debug)]
pub struct AddFadeInOut {
    // elem: Expr, //original region
    pub track_num: usize,
    pub pos: usize,
    pub time_in: f64,
    pub time_out: f64,
}
impl undo::Action for AddFadeInOut {
    type Target = AppModel; //project

    type Output = ();

    type Error = Error;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let mut ctx = target.compile_ctx.parsectx;

        let reg = get_region(target, self.track_num, self.pos)?;
        let time_in = {
            let time = Expr::Literal(Literal::FloatParameter(Arc::new(param_float!(
                self.time_in as f32,
                "time_in",
                0.0..=10.0
            ))));
            ExprRef(ctx.expr_storage.alloc(time))
        };

        let time_out = {
            let time = Expr::Literal(Literal::FloatParameter(Arc::new(param_float!(
                self.time_out as f32,
                "time_in",
                0.0..=10.0
            ))));
            ExprRef(ctx.expr_storage.alloc(time))
        };
        let func = ctx
            .expr_storage
            .alloc(Expr::Var(Symbol(ctx.interner.get_or_intern("fadeinout"))));
        let app = ctx.expr_storage.alloc(Expr::App(
            ExprRef(func),
            vec![reg.clone(), time_in, time_out],
        ));
        *reg = ExprRef(app);
        Ok(())
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
                                match (name.to_string().as_str(), r.as_slice()) {
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
// impl DisplayableAction for AddFadeInOut {}
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
