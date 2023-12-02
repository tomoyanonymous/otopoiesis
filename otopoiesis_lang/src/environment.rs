use super::*;
use std::{
    cell::OnceCell,
    collections::{vec_deque, LinkedList, VecDeque},
};

pub type EnvI = dyn Iterator<Item = Vec<(Symbol, Value)>>;
// pub fn lookup_env<'a, T: Iterator<Item = &'a Vec<(Symbol, Value)>>>(
//     e: &T,
//     symbol: & Symbol,
// ) -> Option<Value> {
//     if let Some(EnvId { level, count }) = symbol.id {
//         e.nth(level as usize)
//             .map(|locals| locals.get(count as usize))
//             .flatten()
//             .map(|v| v.1.clone())
//     } else {
//         let result = e.enumerate().find_map(|(level, locals)| {
//             locals
//                 .iter()
//                 .enumerate()
//                 .find(|(local, (s, v))| s.name == symbol.name)
//                 .map(|v| (level as u64, v))
//         });
//         result.map(|(level, (count, (sym, val)))| {
//             symbol.id = Some(EnvId {
//                 level,
//                 count: count as u64,
//             });
//             val.clone()
//         })
//     }
// }
// pub fn extend_envi(
//     env: &mut VecDeque<Vec<(Symbol, value::Value)>>,
// ) -> vec_deque::Iter<'_, Vec<(Symbol, value::Value)>> {
//     env.push_front(vec![]);
//     env.iter()
// }

pub trait EnvTrait: Clone {
    type Value;
    fn extend_with(&self, kvs: &[(Symbol, Value)]) -> Self;
    fn lookup(&self, key: &Symbol) -> Option<&Self::Value>;
}
type Env = VecDeque<Vec<(Symbol, value::Value)>>;

// pub struct EnvView<'e> {
//     root: &'e mut Env,
//     level: usize,
// }
// impl<'e> EnvView<'e> {
//     pub fn new(root: &'e mut Env) -> Self {
//         Self { root, level: 0 }
//     }

//     pub fn get_or_insert_local_mut(&mut self) -> &mut Vec<(Symbol, value::Value)> {
//         if let Some(ref mut v) = self.root.get_mut(self.level) {
//             v
//         } else {
//             self.root.push_front(vec![]);
//             self.root.get_mut(0).unwrap()
//         }
//     }
// }
// impl<'e> Clone for EnvView<'e> {
//     fn clone(&self) -> Self {
//         EnvView {
//             root: self.root,
//             level: self.level,
//         }
//     }
// }

// impl<'e> EnvTrait for EnvView<'e> {
//     type Value = value::Value;

//     fn lookup(&self, key: &Symbol) -> Option<&Self::Value> {
//         lookup_env(&self.root.range(self.level..), key).as_ref()
//     }

//     fn extend_with(&self, kvs: &[(Symbol, Value)]) -> Self {
//         todo!()
//     }
// }
// pub fn gen_default_env() -> Env {
//     VecDeque::from(Env::default())
// }

// Lexical Environment.
// It is doubley-linked for UI Generation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Environment {
    pub local: Vec<(String, Value)>,
    pub parent: Option<Arc<Self>>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn bind(&mut self, key: &str, val: &Value) {
        self.local.push((key.to_string(), val.clone()))
    }
}
impl Default for Environment {
    fn default() -> Self {
        Self {
            local: vec![],
            parent: None,
        }
    }
}

impl EnvTrait for Arc<Environment> {
    type Value = value::Value;

    fn extend_with(&self, kvs: &[(Symbol, Value)]) -> Self {
        let local = kvs.iter().map(|(s, v)| (s.name.clone(), v.clone())).collect();
        let res = Environment {
            local,
            parent: Some(self.clone()),
        };
        Arc::new(res)
    }

    fn lookup(&self, key: &Symbol) -> Option<&Self::Value> {
        self.local
            .iter()
            .find_map(|e| if e.0 == key.name { Some(&e.1) } else { None })
            .or_else(|| self.parent.as_ref().and_then(|e| e.lookup(key)))
    }
}
