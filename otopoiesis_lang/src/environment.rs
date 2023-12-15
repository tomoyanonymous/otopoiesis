use std::ops::Range;

use super::value::RawValue;
use super::Symbol;
use id_arena::{Arena, Id};
#[derive(Debug)]
pub struct Environment {
    parent: Option<Id<Self>>,
    locals: Range<usize>,
}
pub type EnvironmentRef = Id<Environment>;

#[derive(Debug, Default)]
pub struct EnvironmentStorage<V: Clone + Default> {
    store: Arena<Environment>,
    data: Vec<(Symbol, V)>,
}

impl<V: Clone + Default> EnvironmentStorage<V> {
    pub fn set_root(&mut self, svs: &[(Symbol, V)]) -> Id<Environment> {
        // assert!(self.data.is_empty());
        let range = self.data.len()..(self.data.len() + svs.len());
        self.data.clear();
        svs.iter().for_each(|v| {
            self.data.push(v.clone());
        });
        self.store.alloc(Environment {
            parent: None,
            locals: range,
        })
    }
    pub fn extend(&mut self, parent: Id<Environment>, svs: &[(Symbol, V)]) -> Id<Environment> {
        let range = self.data.len()..(self.data.len() + svs.len());
        self.data
            .resize(range.end, (Symbol::default(), V::default()));
        self.data[range.clone()].clone_from_slice(svs);
        self.store.alloc(Environment {
            parent: Some(parent),
            locals: range,
        })
    }
    pub fn lookup(&self, env: Id<Environment>, key: &Symbol) -> Option<V> {
        let env = self.store.get(env).unwrap();
        let slice = &self.data[env.locals.clone()];
        slice.iter().rev().find_map(|(s, v)| {
            if s == key {
                Some(v.clone())
            } else {
                env.parent.and_then(|penv| self.lookup(penv, key))
            }
        })
    }
}
