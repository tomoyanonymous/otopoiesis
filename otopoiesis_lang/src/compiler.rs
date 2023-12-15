use std::sync::{Arc, Mutex};

use crate::environment::{Environment, EnvironmentRef, EnvironmentStorage};
use crate::expr::{Expr, ExprRef, Literal};
use crate::parameter::{FloatParameter, Parameter, RangedNumeric};
use crate::value::{self, Closure, Project, RawValue, Region, Track};
use crate::Interner;
use crate::{builtin_fn, ExtFun, Symbol};
use id_arena::{Arena, Id};
//unboxed

pub struct Context {
    pub expr_storage: Arena<Expr>,
    pub interner: Interner,
    object_storage: Arena<Closure>,
    pub array_storage: Arena<Vec<RawValue>>,
    pub track_storage: Arena<Track>,
    pub region_storage: Arena<Region>,
    pub project_storage: Arena<Project>,
    pub extfun_storage: Arena<ExtFun>,
    pub env_storage: EnvironmentStorage<RawValue>,
    pub root_env: EnvironmentRef,
}

impl Default for Context {
    fn default() -> Self {
        let expr_storage = Arena::new();
        let interner = Interner::new();
        Self::new(expr_storage, interner)
    }
}

impl Context {
    pub fn new(expr_storage: Arena<Expr>, mut interner: Interner) -> Self {
        let mut extfun_storage = Arena::new();
        let mut env_storage = EnvironmentStorage::default();
        let builtins = builtin_fn::gen_default_functions()
            .iter()
            .map(|(label, fun)| {
                let id = Symbol(interner.get_or_intern(label));
                let fid = extfun_storage.alloc(fun.clone());
                let fref = extfun_storage.get_mut(fid).unwrap();
                let f = RawValue::from(fref as *mut ExtFun);
                (id, f)
            })
            .collect::<Vec<_>>();
        let root_env = env_storage.set_root(&builtins);
        Self {
            expr_storage,
            interner,
            object_storage: Default::default(),
            array_storage: Default::default(),
            track_storage: Default::default(),
            region_storage: Default::default(),
            project_storage: Default::default(),
            extfun_storage,
            env_storage,
            root_env,
        }
    }
    pub fn gen_closure(
        &mut self,
        envid: Id<Environment>,
        ids: &Vec<Symbol>,
        body: &ExprRef,
    ) -> RawValue {
        let cls = Closure::new(envid, ids, body.clone());
        let id = self.object_storage.alloc(cls);
        let ptr = self.object_storage.get_mut(id).unwrap() as *mut Closure;
        RawValue::from(ptr)
    }
    pub fn gen_array(
        &mut self,
        _envid: Id<Environment>,
        values: impl Iterator<Item = RawValue>,
    ) -> RawValue {
        let vecid = self.array_storage.alloc(values.collect());
        let ptr = self.array_storage.get_mut(vecid).unwrap() as *mut Vec<RawValue>;
        RawValue::from(ptr)
    }
    pub fn get_or_intern_str(&mut self, name: &str) -> Symbol {
        Symbol(self.interner.get_or_intern(name))
    }
    fn eval_literal(&self, l: &Literal) -> Result<RawValue, EvalError> {
        match l {
            Literal::Number(v) => Ok(RawValue::from(*v)),
            Literal::FloatParameter(v) => Ok(RawValue::from(v as *const Arc<FloatParameter>)),
            Literal::StringParameter(v) => Ok(RawValue::from(v as *const Arc<Mutex<String>>)),
            Literal::String(_s) => todo!(),
        }
    }
    fn eval_vec(
        &mut self,
        v: &[ExprRef],
        envid: Id<Environment>,
    ) -> Result<Vec<RawValue>, EvalError> {
        v.iter()
            .map(|a| self.eval(a.clone(), envid))
            .try_collect::<Vec<_>>()
    }
    pub fn eval_closure(
        &mut self,
        val: &RawValue,
        args: &[ExprRef],
    ) -> Result<RawValue, EvalError> {
        let Closure { env, ids, body } = val.get_as_ref::<Closure>();
        let argvs = self.eval_vec(args, *env)?;
        let svs = ids
            .into_iter()
            .zip(argvs.into_iter())
            .map(|(id, svs)| (*id, svs))
            .collect::<Vec<_>>();
        let newenv = self.env_storage.extend(*env, svs.as_slice());
        self.eval(body.clone(), newenv)
    }
    pub fn eval(&mut self, e: ExprRef, envid: Id<Environment>) -> Result<RawValue, EvalError> {
        let e = self.expr_storage.get(e.0).ok_or(EvalError::InvalidId)?;

        match e.clone() {
            Expr::Error => Err(EvalError::InvalidConversion),
            Expr::Nop => Err(EvalError::InvalidConversion),
            Expr::Literal(l) => self.eval_literal(&l),
            Expr::Var(sym) => self
                .env_storage
                .lookup(envid, &sym)
                .ok_or(EvalError::NotFound),

            Expr::Let(id, body, then) => {
                let b = self.eval(body.clone(), envid)?;
                let newenv = self.env_storage.extend(envid, &[(id, b)]);
                self.eval(then.clone(), newenv)
            }
            Expr::Then(e1, e2) => {
                let _ = self.eval(e1, envid)?;
                self.eval(e2, envid)
            }

            Expr::Lambda(ids, body) => Ok(self.gen_closure(envid, &ids, &body)),
            Expr::App(callee, args) => {
                let args = self.eval_vec(&args, envid)?;
                let clsv = self.eval(callee, envid)?;
                let Closure { env, ids, body } = clsv.get_as_ref::<Closure>();
                let kvs = ids
                    .into_iter()
                    .zip(args.into_iter())
                    .map(|(id, a)| (*id, a));
                let envid = self.env_storage.extend(*env, &kvs.collect::<Vec<_>>());
                self.eval(body.clone(), envid)
            }
            Expr::BinOp(op, lhs, rhs) => {
                let name = op.get_associated_fn_name();
                //call builtin. todo:lift this before actual eval
                if let Some((_name, fun)) = builtin_fn::gen_default_functions()
                    .iter()
                    .find(|(n, _)| n == name)
                {
                    let lhs = self.eval(lhs, envid)?;
                    let rhs = self.eval(rhs, envid)?;
                    fun.0.exec(&None, self, &[lhs, rhs])
                } else {
                    //if unknown, translate into app
                    let sym = Symbol(self.interner.get_or_intern(name));
                    let var = self.expr_storage.alloc(Expr::Var(sym));
                    let app = ExprRef(
                        self.expr_storage
                            .alloc(Expr::App(ExprRef(var), vec![lhs, rhs])),
                    );
                    self.eval(app, envid)
                }
            }
            Expr::AppExt(callee, args) => {
                let args = self.eval_vec(&args, envid)?;
                let f = unsafe { callee.as_mut() }.unwrap();
                f.0.exec(&None, self, &args)
            }

            Expr::Array(es) => {
                let vs = self.eval_vec(&es, envid)?;
                Ok(self.gen_array(envid, vs.into_iter()))
            }
            Expr::Track(e) => {
                let id = self.track_storage.alloc(Track(envid, e));
                let ptr = self.track_storage.get_mut(id).unwrap() as *mut Track;
                Ok(RawValue::from(ptr))
            }
            Expr::Region(start, dur, content) => {
                let id = self
                    .region_storage
                    .alloc(Region(envid, start, dur, content));
                let ptr = self.region_storage.get_mut(id).unwrap() as *mut Region;
                Ok(RawValue::from(ptr))
            }
            Expr::Project(sr, es) => {
                let id = self
                    .project_storage
                    .alloc(value::Project(envid.clone(), sr, es.clone()));
                let ptr = self.project_storage.get_mut(id).unwrap() as *mut Project;
                Ok(RawValue::from(ptr))
            }
            Expr::Paren(e) | Expr::Block(e) => self.eval(e, envid),
            Expr::WithAttribute(attr, e) => {
                let expr = self.expr_storage.get(e.0).unwrap();
                match (self.interner.resolve(attr.0 .0), expr.clone()) {
                    (Some("param"), Expr::Let(name, body, then)) => {
                        let b = self.expr_storage.get(body.0).unwrap();
                        let body = match b {
                            Expr::Literal(Literal::Number(f)) => {
                                let label = self.interner.resolve(name.0).unwrap();
                                let lit = Arc::new(
                                    FloatParameter::new(*f as f32, label)
                                        .set_range(*attr.1.start() as f32..=*attr.1.end() as f32),
                                );
                                let fname = self.interner.get_or_intern("param_as_number");
                                let extfun = self
                                    .env_storage
                                    .lookup(envid, &Symbol(fname))
                                    .map(|rv| rv.get_as_mut_ptr::<ExtFun>())
                                    .unwrap();
                                let param = Expr::Literal(Literal::FloatParameter(lit));
                                let paramr = ExprRef(self.expr_storage.alloc(param));
                                Expr::AppExt(extfun, vec![paramr])
                            }
                            _ => b.clone(),
                        };
                        let body = ExprRef(self.expr_storage.alloc(body));
                        let newexpr =
                            ExprRef(self.expr_storage.alloc(Expr::Let(name, body, then.clone())));
                        self.eval(newexpr, envid)
                    }
                    _ => self.eval(e, envid),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalError {
    TypeMismatch(String),
    NotFound,
    InvalidNumArgs(usize, usize), //expected,actual
    InvalidId,
    InvalidConversion,
    NotInPlayMode,
    NoAppRuntime,
}
