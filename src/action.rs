//! The definitions of un/redo-able actions for applications.
//! Todo: fix an inconsistensy after code-app translation because serializing/deserializing refreshes Arc references.



use crate::data::{self, SharedVec};
use std::sync::{Arc, MutexGuard, PoisonError};
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

// pub struct AddtoContainer<T: Clone>{
//     elem_to_add:T,
//     container:Vec<T>
// };
// trait ApptoContainer<T> {
//     fn get_target_container_from_app(&self, app: Arc<Mutex<data::AppModel>>) -> MutexGuard<Vec<T>>;
// }
pub struct OpsContainer<T: Clone> {
    container: SharedVec<T>,
    elem_to_add: Option<T>,
}
impl<T: Clone> OpsContainer<T> {
    fn add(&self) -> Result<(), OpsContainerError> {
        let mut c = self.container.lock().unwrap();
        match &self.elem_to_add {
            Some(e) => {
                c.push(e.clone());
                Ok(())
            }
            None => Err(OpsContainerError::NothingToBeAdded),
        }
    }
    fn remove(&mut self, memory: bool) -> Result<(), OpsContainerError> {
        let mut c = self.container.lock().unwrap();
        match c.pop() {
            Some(e) => {
                if memory {
                    self.elem_to_add = Some(e);
                }
                Ok(())
            }
            None => Err(OpsContainerError::ContainerEmpty),
        }
    }
}
struct AddtoContainer<T: Clone>(OpsContainer<T>);
struct RemovefromContainer<T: Clone>(OpsContainer<T>);

impl<T> undo::Action for AddtoContainer<T>
where
    T: Clone + std::fmt::Display,
{
    type Target = (); //target will be managed on action side
    type Output = ();
    type Error = OpsContainerError;
    fn apply(&mut self, _target: &mut Self::Target) -> undo::Result<Self> {
        self.0.add()
    }
    fn undo(&mut self, _target: &mut Self::Target) -> undo::Result<Self> {
        self.0.remove(false)
    }
}

impl<T> undo::Action for RemovefromContainer<T>
where
    T: Clone + std::fmt::Display,
{
    type Target = ();
    type Output = ();
    type Error = OpsContainerError;
    fn apply(&mut self, _target: &mut Self::Target) -> undo::Result<Self> {
        self.0.remove(true)
    }
    fn undo(&mut self, _target: &mut Self::Target) -> undo::Result<Self> {
        self.0.add()
    }
}
impl<T> std::fmt::Display for AddtoContainer<T>
where
    T: Clone + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Add {}",
            self.0
                .elem_to_add
                .as_ref()
                .map_or("".to_string(), |v| v.to_string())
        )
    }
}
impl<T> std::fmt::Display for RemovefromContainer<T>
where
    T: Clone + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Remove {}",
            self.0
                .elem_to_add
                .as_ref()
                .map_or("".to_string(), |v| v.to_string())
        )
    }
}
trait DisplayableAction: undo::Action + std::fmt::Display {}
impl<T> DisplayableAction for AddtoContainer<T>
where
    T: Clone,
    AddtoContainer<T>: undo::Action + std::fmt::Display,
{
}
impl<T> DisplayableAction for RemovefromContainer<T>
where
    T: Clone,
    RemovefromContainer<T>: undo::Action + std::fmt::Display,
{
}

pub struct Action(
    Box<dyn DisplayableAction<Target = (), Output = (), Error = OpsContainerError> + 'static>,
);
pub enum Target {
    Tracks(data::Project),
    Regions(SharedVec<Arc<data::Region>>),
}

impl undo::Action for Action {
    type Target = ();
    type Output = ();
    type Error = OpsContainerError;
    fn apply(&mut self, _target: &mut ()) -> undo::Result<Self> {
        self.0.apply(_target)
    }
    fn undo(&mut self, _target: &mut ()) -> undo::Result<Self> {
        self.0.undo(_target)
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn make_action_dyn(
    a: impl DisplayableAction<Target = (), Output = (), Error = OpsContainerError> + 'static,
) -> Action {
    Action(Box::new(a))
}

pub fn add_region(
    app: &mut data::AppModel,
    track: SharedVec<Arc<data::Region>>,
    region: Arc<data::Region>,
) -> Result<(), OpsContainerError> {
    // let action = AddtoContainer::<Arc<data::Region>>(region.clone())
    app.history.apply(
        &mut (),
        make_action_dyn(AddtoContainer::<Arc<data::Region>>(OpsContainer {
            container: track,
            elem_to_add: Some(region),
        })),
    )
}
pub fn add_track(app: &mut data::AppModel, track: data::Track) -> Result<(), OpsContainerError> {
    app.history.apply(
        &mut (),
        make_action_dyn(AddtoContainer::<data::Track>(OpsContainer {
            container: Arc::clone(&app.project.tracks),
            elem_to_add: Some(track),
        })),
    )
}
