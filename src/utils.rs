//! Misc utilities such as Atomic Structure.
pub mod atomic;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::parameter::{FloatParameter, Parameter};

pub use self::atomic::{make_simple_atomic, SimpleAtomic, SimpleAtomicTest};
use atomic::IsAtomicNumber;

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct AtomicRange {
    start: Arc<FloatParameter>,
    dur: Arc<FloatParameter>,
}

impl AtomicRange {
    pub fn new(start: Arc<FloatParameter>, dur: Arc<FloatParameter>) -> Self {
        Self { start, dur }
    }

    pub fn start(&self) -> f32 {
        self.start.get()
    }
    pub fn end(&self) -> f32 {
        self.start()+self.dur.get()
    }
    pub fn getrange(&self) -> f32 {
        self.dur.get()
    }
    pub fn contains(&self, v: f32) -> bool {
        (self.start()..=self.end()).contains(&v)
    }
    pub fn set_start(&self, v: f32) {
        self.start.set(v);
    }
    pub fn set_end(&self, v: f32) {
        self.dur.set(v-self.start());
    }
    pub fn shift(&self, v: f32) {
        self.set_start(self.start() + v);
        self.set_end(self.end() + v);
    }
    //does not shrink when the range reached to 0.
    pub fn shift_bounded(&self, v: f32) {
        let mut start_bounded = self.start() + v;
        if start_bounded > 0.0 {
            start_bounded = 0.0;
        }
        let end_bounded = start_bounded + self.getrange();
        self.set_start(start_bounded.max(0.0));
        self.set_end(end_bounded);
    }
}

