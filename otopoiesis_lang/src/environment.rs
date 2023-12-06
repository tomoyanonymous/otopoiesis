use std::ops::Range;

use super::value::RawValue;
use super::Symbol;
use id_arena::{Arena, Id};
use string_interner::StringInterner;
pub struct Environment {
    parent: Option<Id<Self>>,
    locals: Range<usize>,
}
pub type EnvironmentRef = Id<Environment>;

#[derive(Default)]
pub struct EnvironmentStorage {
    store: Arena<Environment>,
    data: Vec<(Symbol, RawValue)>,
}

impl EnvironmentStorage {

    pub fn set_root(&mut self, svs: &[(Symbol, RawValue)]) {
        assert!(self.data.is_empty());
        let range = self.data.len()..(self.data.len() + svs.len());
        self.data.clear();
        svs.iter().for_each(|v|{
            self.data.push(*v);
        });
        self.store.alloc(Environment {
            parent: None,
            locals: range,
        });
    }
    pub fn extend(
        &mut self,
        parent: Id<Environment>,
        svs: &[(Symbol, RawValue)],
    ) -> Id<Environment> {
        let range = self.data.len()..(self.data.len() + svs.len());
        self.data.clone_from_slice(svs);
        self.store.alloc(Environment {
            parent: Some(parent),
            locals: range,
        })
    }
    pub fn lookup(&self, env: Id<Environment>, key: &Symbol) -> Option<RawValue> {
        let env = self.store.get(env).unwrap();
        let slice = &self.data[env.locals.clone()];
        slice.iter().rev().find_map(|(s, v)| {
            if s == key {
                Some(*v)
            } else {
                env.parent.and_then(|penv| self.lookup(penv, key))
            }
        })
    }
}
