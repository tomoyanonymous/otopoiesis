// lock free vector, which can be updated from gui thread without lock by copying another vector locked with mutex before the playback.
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

pub struct LockFreeVec<T> {
    content_rt: Vec<T>,
    content: Arc<Mutex<Vec<T>>>,
}

impl<T> LockFreeVec<T>
where
    T: std::default::Default + Copy,
{
    pub fn new(size: usize) -> Self {
        Self {
            content_rt: vec![T::default(); size],
            content: Arc::new(Mutex::new(vec![T::default(); size])),
        }
    }
    pub fn modify<'a, F>(&self, update: F)
    where
        F: Fn(&mut Vec<T>) + 'a,
    {
        let lock = self.content.try_lock();
        if let Ok(mut v) = lock {
            update(v.deref_mut())
        }
    }
    pub fn copy_to_rt(&mut self) {
        let lock = self.content.try_lock();
        if let Ok(v) = lock {
            self.content_rt.clone_from(v.deref())
        }
    }
    pub fn copy_from_rt(&mut self) {
        let lock = self.content.try_lock();
        if let Ok(mut v) = lock {
            v.clone_from(&self.content_rt)
        }
    }

    
    pub fn get_rt(&self) -> &Vec<T> {
        &self.content_rt
    }
    pub fn get_rt_mut(&mut self) -> &mut Vec<T> {
        &mut self.content_rt
    }
}
