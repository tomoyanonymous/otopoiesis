pub mod lockfree_container;

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
pub struct AtomicRange(pub AtomicU64, pub AtomicU64);

type EguiGetSet<'a> = Box<dyn FnMut(Option<f64>) -> f64 + 'a>;

impl AtomicRange {
    pub fn new(start: u64, end: u64) -> Self {
        Self(AtomicU64::from(start), AtomicU64::from(end))
    }
    pub fn get_pair(&self) -> (u64, u64) {
        (self.start(), self.end())
    }
    pub fn start(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
    pub fn end(&self) -> u64 {
        self.1.load(Ordering::Relaxed)
    }
    pub fn getrange(&self) -> u64 {
        &self.1.load(Ordering::Relaxed) - &self.0.load(Ordering::Relaxed)
    }
    pub fn contains(&self, v: u64) -> bool {
        let (min, max) = self.get_pair();
        (min..max).contains(&v)
    }
    pub fn set_start(&self, v: u64) {
        self.0.store(v, Ordering::Relaxed);
    }
    pub fn set_end(&self, v: u64) {
        self.1.store(v, Ordering::Relaxed);
    }

    pub fn egui_get_set_start(&self, scaling_factor: f64) -> EguiGetSet {
        Box::new(move |f: Option<f64>| -> f64 {
            if let Some(v) = f {
                self.set_start((v * scaling_factor) as u64);
            }
            let res = self.get_pair().0 as f64 / scaling_factor;
            res
        })
    }
    pub fn egui_get_set_end(&self, scaling_factor: f64) -> EguiGetSet {
        Box::new(move |f: Option<f64>| -> f64 {
            if let Some(v) = f {
                self.set_end((v * scaling_factor) as u64);
            }
            let res = self.get_pair().1 as f64 / scaling_factor;
            res
        })
    }
    
}
impl Clone for AtomicRange{
    fn clone(&self) -> Self {
        Self::new(self.start(),self.end())
    }
}