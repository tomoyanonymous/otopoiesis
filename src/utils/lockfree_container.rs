// lock free vector, which can be updated from gui thread without lock by copying another vector locked with mutex before the playback.
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "Vec<T>", into = "Vec<T>")]
pub struct LockFreeVec<T>
where
    T: Clone,
{
    content_rt: RefCell<Vec<T>>,
    content: Arc<Mutex<Vec<T>>>,
}

impl<T> LockFreeVec<T>
where
    T: std::default::Default + Clone,
{
    pub fn new(size: usize) -> Self {
        Self {
            content_rt: RefCell::new(vec![T::default(); size]),
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
    //this is essentially mutable but protected with refcell
    pub fn copy_to_rt(&self) {
        let lock = self.content.try_lock();
        if let Ok(v) = lock {
            self.content_rt.borrow_mut().clone_from(v.deref())
        }
    }
    pub fn copy_from_rt(&self) {
        let lock = self.content.try_lock();
        if let Ok(mut v) = lock {
            v.clone_from(&self.content_rt.borrow())
        }
    }

    pub fn get_rt(&self) -> RefCell<Vec<T>> {
        self.content_rt.clone()
    }

}

impl<T> From<Vec<T>> for LockFreeVec<T>
where
    T: Clone,
{
    fn from(t: Vec<T>) -> Self {
        let t_clone = t.clone();
        LockFreeVec::<T> {
            content_rt: RefCell::new(t),
            content: Arc::new(Mutex::new(t_clone)),
        }
    }
}
impl<T> Into<Vec<T>> for LockFreeVec<T>
where
    T: Clone,
{
    fn into(self) -> Vec<T> {
        self.content_rt.borrow().clone()
    }
}

// impl<T> Serialize for LockFreeVec<T>
// where
//     T: Serialize,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let seq = serializer.serialize_seq(Some(self.content_rt.len()))?;
//         for element in self.content_rt.iter() {
//             seq.serialize_element(element);
//         }
//         seq.end()
//     }
// }
// struct Visitor;

// impl<'de, T> serde::de::Visitor<'de> for LockFreeVec<T>
// where
//     T: Serialize,
//     Self: Sized,
// {
//     type Value = T;
//     fn
// }
// impl<'de, T> Deserialize<'de> for LockFreeVec<T>
// where
//     T: Deserialize,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {

//         let seq = deserializer.deserialize_seq(Visitor {})?;
//         seq
//     }
//     fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
//         where
//             D: serde::Deserializer<'de>, {
//                 deserializer.
//     }
// }
