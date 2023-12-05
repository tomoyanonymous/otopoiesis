use std::sync::Mutex;

use super::{Symbol, *};

#[derive(Debug, Clone, PartialEq)]
pub struct ExprRef(pub id_arena::Id<Expr>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Literal {
    Number(f64),
    FloatParameter(Arc<FloatParameter>),
    StringParameter(Arc<Mutex<String>>),
    String(String),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pattern {
    label: String,
}
#[derive(Debug, Clone)]
pub enum Expr {
    Nop,
    Literal(Literal),
    Array(Vec<ExprRef>),
    Var(Symbol),
    Let(Symbol, ExprRef, ExprRef),
    App(ExprRef, Vec<ExprRef>), //currently only single argument
    AppExt(ExtFun, Vec<ExprRef>),
    Lambda(Vec<Symbol>, ExprRef),
    //track and region is an alias to closure
    Track(ExprRef),
    Region(ExprRef, ExprRef, ExprRef), //start,dur,content
    Project(f64, Vec<ExprRef>),
}
