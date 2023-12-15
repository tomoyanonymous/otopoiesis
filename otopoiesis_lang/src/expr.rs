use std::{ops::RangeInclusive, sync::Mutex};

use crate::parser::{ParseContext, ParseContextRef};

use super::{Symbol, *};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    Sum,     // +
    Minus,   // -
    Product, // *
    Divide,  // /

    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=

    Modulo,   // %
    Exponent, // ^

    And, // &&
    Or,  // ||

    Pipe, // |>
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprRef(pub id_arena::Id<Expr>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Literal {
    Number(f64),
    FloatParameter(Arc<FloatParameter>),
    StringParameter(Arc<Mutex<String>>),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute(pub Symbol, pub RangeInclusive<f64>);
impl Attribute {
    pub fn stringify(
        &self,
        ctx: &ParseContext,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let key = ctx.interner.resolve(self.0.0).unwrap_or_default();
        let start = self.1.start();
        let end = self.1.end();
        write!(f, "#[{key}({start}..{end})]")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pattern {
    label: String,
}
#[derive(Debug, Clone)]
pub enum Expr {
    Nop,
    Error,
    Literal(Literal),
    Array(Vec<ExprRef>),
    Var(Symbol),
    Let(Symbol, ExprRef, ExprRef),
    App(ExprRef, Vec<ExprRef>),  //currently only single argument
    BinOp(Op, ExprRef, ExprRef), //semantically identical to App
    AppExt(*mut ExtFun, Vec<ExprRef>),
    Lambda(Vec<Symbol>, ExprRef),
    Paren(ExprRef), //semantically meaningless, just for inverse evaluation
    WithAttribute(Attribute, ExprRef),
    //track and region is an alias to closure
    Track(ExprRef),
    Region(ExprRef, ExprRef, ExprRef), //start,dur,content
    Project(f64, Vec<ExprRef>),

}
