use id_arena::{Arena, Id};

use crate::{
    environment::EnvironmentStorage,
    error::ReportableError,
    expr::{ExprRef, Literal},
    metadata::Span,
    parser::ParseContext,
    types::{Type, TypeId},
    Environment, Expr, Interner,
};
use std::collections::BTreeMap;
use std::rc::Rc;

pub type TypeRef = Type;

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    TypeMismatch,
    CircularType,
    IndexOutOfRange(u16, u16),
    IndexForNonTuple,
    VariableNotFound(String),
    NonPrimitiveInFeed,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Error(ErrorKind, Span);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            ErrorKind::TypeMismatch => write!(f, "Type Mismatch"),
            ErrorKind::CircularType => write!(f, "Circular loop of type definition"),
            ErrorKind::IndexOutOfRange(len, idx) => write!(
                f,
                "Length of tuple elements is {} but index was {}",
                len, idx
            ),
            ErrorKind::IndexForNonTuple => write!(f, "Index access for non-tuple variable"),
            ErrorKind::VariableNotFound(v) => {
                write!(f, "Variable {} not found in this scope", v)
            }
            ErrorKind::NonPrimitiveInFeed => {
                write!(f, "Function that uses self cannot be return function type.")
            }
        }
    }
}
impl std::error::Error for Error {}
impl ReportableError for Error {
    fn get_span(&self) -> std::ops::Range<usize> {
        self.1.clone()
    }
}
#[derive(Debug)]
pub struct InferContext {
    interm_idx: i64,
    subst_map: BTreeMap<i64, Type>,
    interner: Interner,
    expr_storage: Arena<Expr>,
    pub span_storage: BTreeMap<Id<Expr>, Span>,
    pub envstorage: EnvironmentStorage<Type>,
}
impl InferContext {
    pub fn new(parsectx: ParseContext) -> Self {
        let ParseContext {
            expr_storage,
            span_storage,
            interner,
        } = parsectx;
        Self {
            interm_idx: 0,
            subst_map: BTreeMap::<i64, Type>::new(),
            interner,
            expr_storage,
            span_storage,
            envstorage: EnvironmentStorage::<Type>::default(),
        }
    }
    pub fn gen_intermediate_type(&mut self) -> Type {
        let res = Type::Intermediate(self.interm_idx);
        self.interm_idx += 1;
        res
    }
    // return true when the circular loop of intermediate variable exists.
    pub fn occur_check(&self, id1: i64, t2: Type) -> bool {
        let cls = |t2dash: Type| -> bool { self.occur_check(id1, t2dash) };
        let vec_cls = |t: Vec<_>| -> bool { t.iter().all(|a: &Type| cls(a.clone())) };
        match t2 {
            Type::Intermediate(id2) => {
                if id1 == id2 {
                    true
                } else {
                    self.subst_map
                        .get(&id1)
                        .map_or(false, |r| self.occur_check(id1, r.clone()))
                }
            }
            Type::Array(a, _l) => cls(*a),
            Type::Tuple(t) => vec_cls(t),
            Type::Function(p, r) => vec_cls(p) && cls(*r),
            // Type::Struct(_s) => todo!(),
            _ => false,
        }
    }
    fn substitute_intermediate_type(&self, id: i64) -> Option<Type> {
        match self.subst_map.get(&id) {
            Some(t) => match t {
                Type::Intermediate(i) => self.substitute_intermediate_type(*i),
                _ => Some(t.clone()),
            },
            None => None,
        }
    }
    pub fn substitute_type(&self, t: Type) -> Option<Type> {
        match t {
            Type::Intermediate(id) => self.substitute_intermediate_type(id),
            _ => Some(t.apply_fn(|e: Type| self.substitute_type(e).unwrap_or(Type::Unknown))),
        }
    }
    fn unify_types(&mut self, t1: Type, t2: Type) -> Result<Type, ErrorKind> {
        let mut unify_vec = |a1: Vec<Type>, a2: Vec<Type>| -> Result<Vec<_>, ErrorKind> {
            a1.clone()
                .iter()
                .zip(a2.iter())
                .map(|(v1, v2)| self.unify_types(v1.clone(), v2.clone()))
                .collect()
        };
        match (t1.clone(), t2.clone()) {
            (Type::Unit, Type::Unit) => Ok(Type::Unit),
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Number, Type::Number) => Ok(Type::Number),
            (Type::String, Type::String) => Ok(Type::String),

            (Type::Intermediate(i1), Type::Intermediate(i2)) => {
                if self.occur_check(i1, t2.clone()) {
                    Err(ErrorKind::CircularType)
                } else {
                    if i1 < i2 {
                        self.subst_map.insert(i1, t2);
                        Ok(t1)
                    } else {
                        self.subst_map.insert(i2, t1);
                        Ok(t2)
                    }
                }
            }
            (Type::Intermediate(i), t) | (t, Type::Intermediate(i)) => {
                self.subst_map.insert(i, t.clone());
                Ok(t)
            }

            (Type::Array(box a1, l1), Type::Array(box a2, _)) => {
                Ok(Type::Array(Box::new(self.unify_types(a1, a2)?), l1))
            }
            (Type::Tuple(a1), Type::Tuple(a2)) => Ok(Type::Tuple(unify_vec(a1, a2)?)),
            // (Type::Struct(_a1), Type::Struct(_a2)) => todo!(), //todo
            (Type::Function(p1, box r1), Type::Function(p2, box r2)) => Ok(Type::Function(
                unify_vec(p1, p2)?,
                Box::new(self.unify_types(r1, r2)?),
            )),
            (Type::Code(box p1), Type::Code(box p2)) => {
                Ok(Type::Code(Box::new(self.unify_types(p1, p2)?)))
            }
            (_p1, _p2) => Err(ErrorKind::TypeMismatch),
        }
    }
}
fn infer_type_literal(e: &Literal) -> Type {
    let pt = match e {
        Literal::Number(_s) => Type::Number,
        // Literal::Int(_s) => Type::Int,
        Literal::String(_s) => Type::String,
        Literal::FloatParameter(_) => Type::Number,
        Literal::StringParameter(_) => Type::String,
        // Literal::Now => PType::Numeric,
        // Literal::SelfLit => panic!("\"self\" should not be shown at type inference stage"),
    };
    pt
}

pub fn infer_type(
    eid: &ExprRef,
    env: Id<Environment>,
    ctx: &mut InferContext,
) -> Result<Type, Error> {
    let infer_vec = |e: &Vec<ExprRef>, ctx: &mut InferContext| {
        e.iter()
            .map(|el| Ok(infer_type(el, env, ctx)?))
            .collect::<Result<Vec<_>, Error>>()
    };
    let e = ctx.expr_storage.get(eid.0).unwrap().clone();
    match e {
        Expr::Literal(l) => Ok(infer_type_literal(&l)),
        // Expr::Tuple(e) => Ok(Type::Tuple(infer_vec(e, ctx)?)),
        // Expr::Proj(e, idx) => {
        //     let tup = infer_type(&e.0, ctx)?;
        //     match tup {
        //         Type::Tuple(vec) => {
        //             if vec.len() < *idx as usize {
        //                 Err(Error(
        //                     ErrorKind::IndexOutOfRange(vec.len() as u16, *idx as u16),
        //                     e.1.clone(),
        //                 ))
        //             } else {
        //                 Ok(vec[*idx as usize].clone())
        //             }
        //         }
        //         _ => Err(Error(ErrorKind::IndexForNonTuple, e.1.clone())),
        //     }
        // }
        // Expr::Feed(id, body) => {
        //     ctx.env.extend();
        //     let feedv = ctx.gen_intermediate_type();
        //     ctx.env.add_bind(&mut vec![(id.clone(), feedv.clone())]);
        //     let b = infer_type(&body.0, ctx);
        //     let res = ctx.unify_types(b?, feedv)?;
        //     ctx.env.to_outer();
        //     if res.is_primitive() {
        //         Ok(res)
        //     } else {
        //         Err(Error(ErrorKind::NonPrimitiveInFeed, body.1))
        //     }
        // }
        Expr::Lambda(p, body) => {
            let infer_params = |e: &Vec<_>, c: &mut InferContext| {
                e.iter().map(|_id| c.gen_intermediate_type()).collect()
            };

            let bty = infer_type(&body, env, ctx)?;
            Ok(Type::Function(infer_params(&p, ctx), Box::new(bty)))
        }
        Expr::Let(id, body, then) => {
            let bodyt = infer_type(&body, env, ctx)?;
            let idt = ctx.gen_intermediate_type();
            let bodyt_u = ctx
                .unify_types(idt, bodyt)
                .map_err(|kind| Error(kind, ctx.span_storage.get(&eid.0).unwrap().clone()))?;
            ctx.envstorage.extend(env, &[(id, bodyt_u)]);
            infer_type(&then, env, ctx)
        }
        // Expr::LetTuple(_ids, _body, _then) => {
        //     todo!("should be de-sugared before type inference")
        // }
        // Expr::LetRec(id, body, then) => {
        //     let c = ctx;
        //     let idt = id.ty.unwrap_or(c.gen_intermediate_type());
        //     c.env.extend();
        //     let body_i = c.gen_intermediate_type();
        //     c.env.add_bind(&mut vec![(id.id, body_i)]);
        //     let bodyt = infer_type(&body.0, c)?;
        //     let _ = c.unify_types(idt, bodyt)?;

        //     let res = match then {
        //         Some(e) => infer_type(&e.0, c),
        //         None => Ok(Type::Primitive(PType::Unit)),
        //     };
        //     c.env.to_outer();
        //     res
        // }
        Expr::Var(name) => {
            let namestr = ctx.interner.resolve(name.0).unwrap();
            ctx.envstorage.lookup(env, &name).map_or(
                Err(Error(
                    ErrorKind::VariableNotFound(namestr.to_string()),
                    0..0,
                )), //todo:Span
                |v| Ok(v.clone()),
            )
        }
        Expr::App(fun, callee) => {
            let fnl = infer_type(&fun, env, ctx)?;
            let callee_t = infer_vec(&callee, ctx)?;
            let res_t = ctx.gen_intermediate_type();
            let fntype = Type::Function(callee_t, Box::new(res_t));
            ctx.unify_types(fnl, fntype)
                .map_err(|kind| Error(kind, ctx.span_storage.get(&eid.0).unwrap().clone()))
        }
        // Expr::If(cond, then, opt_else) => {
        //     let condt = infer_type(&cond.0, ctx)?;
        //     let _bt = ctx.unify_types(Type::Primitive(PType::Int), condt); //todo:boolean type
        //     let thent = infer_type(&then.0, ctx)?;
        //     let elset =
        //         opt_else.map_or(Ok(Type::Primitive(PType::Unit)), |e| infer_type(&e.0, ctx))?;
        //     ctx.unify_types(thent, elset)
        // }
        // Expr::(expr) => {
        //     expr.map_or(Ok(Type::Primitive(PType::Unit)), |e| infer_type(&e.0, ctx))
        // }
        _ => {
            // todo!();
            Ok(Type::Unknown)
        }
    }
}
pub fn infer_type_recovery(
    eid: &ExprRef,
    env: Id<Environment>,
    ctx: &mut InferContext,
) -> (Type, Vec<Error>) {
    let e = ctx.expr_storage.get(eid.0).unwrap().clone();
    let span = ctx.span_storage.get(&eid.0).unwrap().clone();
    let mut errs = vec![];
    let t = match e {
        Expr::Nop => Type::Unit,
        Expr::Error => Type::Error,
        Expr::Literal(l) => infer_type_literal(&l),
        Expr::Array(_) => todo!(),
        Expr::Var(name) => {
            let namestr = ctx.interner.resolve(name.0).unwrap();
            ctx.envstorage.lookup(env, &name).unwrap_or_else(
                || {
                    errs.push(Error(
                        ErrorKind::VariableNotFound(namestr.to_string()),
                        span,
                    ));
                    Type::Error
                }, //todo:Span
            )
        }
        Expr::Let(id, body, then) => {
            let (bodyt, mut es) = infer_type_recovery(&body, env, ctx);
            errs.append(&mut es);
            let idt = ctx.gen_intermediate_type();
            let bodyt_u = ctx
                .unify_types(idt, bodyt)
                .map_err(|kind| Error(kind, span));
            let bodyt_res = match bodyt_u {
                Ok(t) => t,
                Err(e) => {
                    errs.push(e);
                    Type::Error
                }
            };
            ctx.envstorage.extend(env, &[(id, bodyt_res)]);
            let (thent, mut es) = infer_type_recovery(&then, env, ctx);
            errs.append(&mut es);
            thent
        }
        Expr::Then(_, _) => todo!(),
        Expr::App(_, _) => todo!(),
        Expr::BinOp(_, _, _) => todo!(),
        Expr::AppExt(_, _) => todo!(),
        Expr::Lambda(_, _) => todo!(),
        Expr::Block(_) => todo!(),
        Expr::Paren(_) => todo!(),
        Expr::WithAttribute(_, _) => todo!(),
        Expr::Track(_) => todo!(),
        Expr::Region(_, _, _) => todo!(),
        Expr::Project(_, _) => todo!(),
    };
    (t, errs)
}
