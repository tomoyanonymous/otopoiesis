use std::ops::Range;

use id_arena::{Arena, Id};
use string_interner::{DefaultSymbol, StringInterner};
type Symbol = DefaultSymbol;
type ExprRef = Id<Expr>;
//unboxed
type RawStorage = u128;
#[derive(Clone, Copy, Debug)]
pub struct RawValue(u128);

pub struct Closure {
    env: Id<Environment>,
    ids: Vec<Symbol>,
    body: ExprRef,
}

pub enum Expr {
    Number(f64),
    Var(Symbol),
    Let(Symbol, ExprRef, ExprRef),
    Lambda(Vec<Symbol>, ExprRef),
    App(ExprRef, Vec<ExprRef>),
}

pub struct Environment {
    parent: Option<Id<Self>>,
    locals: Range<usize>,
}
struct EnvironmentStorage {
    store: Arena<Environment>,
    data: Vec<(Symbol, RawValue)>,
}

impl EnvironmentStorage {
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

pub struct Compiler {
    expr_storage: Arena<Expr>,
}
pub struct Context {
    interner: StringInterner,
    object_storage: Arena<Closure>,
    env_storage: EnvironmentStorage,
}
impl Context {
    pub fn get_or_intern_str(&mut self, name: &str) -> Symbol {
        self.interner.get_or_intern(name)
    }
}

pub enum Error {
    InvalidId,
    NotFound,
}
impl Compiler {
    pub fn eval(
        &self,
        e: ExprRef,
        envid: Id<Environment>,
        ctx: &mut Context,
    ) -> Result<RawValue, Error> {
        assert_eq!(std::mem::align_of::<Id<Closure>>(), 16);
        let e = self.expr_storage.get(e).ok_or(Error::InvalidId)?;
        match e {
            Expr::Number(f) => {
                let res = unsafe { std::mem::transmute_copy::<f64, RawValue>(&f) };
                Ok(res)
            }
            Expr::Var(sym) => ctx.env_storage.lookup(envid, sym).ok_or(Error::NotFound),

            Expr::Let(id, body, then) => {
                let b = self.eval(*body, envid, ctx)?;
                let newenv = ctx.env_storage.extend(envid, &[(*id, b)]);
                self.eval(*then, newenv, ctx)
            }
            Expr::Lambda(ids, body) => {
                let cls = ctx.object_storage.alloc(Closure {
                    env: envid,
                    ids: ids.clone(),
                    body: body.clone(),
                });
                Ok(unsafe { std::mem::transmute_copy::<Id<Closure>, RawValue>(&cls) })
            }
            Expr::App(callee, args) => {
                let args = args
                    .iter()
                    .map(|a| self.eval(*a, envid, ctx))
                    .try_collect::<Vec<_>>()?;
                let clsid = unsafe {
                    std::mem::transmute_copy::<RawValue, Id<Closure>>(
                        &self.eval(*callee, envid, ctx)?,
                    )
                };
                let Closure { env, ids, body } =
                    ctx.object_storage.get(clsid).ok_or(Error::InvalidId)?;
                let kvs = ids
                    .into_iter()
                    .zip(args.into_iter())
                    .map(|(id, a)| (*id, a));
                let envid = ctx.env_storage.extend(*env, &kvs.collect::<Vec<_>>());
                self.eval(*body, envid, ctx)
            }
        }
    }
}
