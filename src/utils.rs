//! Misc utilities such as Atomic Structure.
pub mod atomic;
pub use atomic::{make_simple_atomic, SimpleAtomic, SimpleAtomicTest,AtomicRange};

pub(crate) mod logger;
pub(crate) use logger::{Logger,GLOBAL_LOGGER};
