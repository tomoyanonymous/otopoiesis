use super::ParseContext;
use crate::{
    expr::{ExprRef, Literal},
    Expr, Symbol,
};
use std::fmt::Display;

pub struct Stringifier<'a> {
    ctx: &'a ParseContext,
    level: u32,
    e: ExprRef,
}

impl<'a> Stringifier<'a> {
    const DELIMITER: &'static str = ",";
    const INDENT: &'static str = "  ";
    pub fn new(ctx: &'a ParseContext, level: u32, e: ExprRef) -> Self {
        Self { ctx, level, e }
    }
    fn get_symbol_str(&self, sym: &Symbol) -> &str {
        self.ctx.interner.resolve(*sym).unwrap_or("")
    }
    fn stringify(&self, e: &ExprRef) -> Self {
        Stringifier::<'a>::new(self.ctx, self.level, e.clone())
    }
    fn stringify_indent(&self, e: &ExprRef) -> Self {
        Stringifier::<'a>::new(self.ctx, self.level + 1, e.clone())
    }
    fn join_symbols(&self, syms: impl Iterator<Item = &'a Symbol>) -> String {
        syms.map(|id| self.ctx.interner.resolve(*id).unwrap())
            .fold("".to_string(), |acc, id| {
                if acc.is_empty() {
                    id.into()
                } else {
                    format!("{acc}{}{id}", Self::DELIMITER)
                }
            })
    }
    fn write_indent(&self, f: &mut std::fmt::Formatter<'_>) {
        (0..self.level).into_iter().for_each(|_| {
            let _ = write!(f, "{}", Self::INDENT);
        })
    }
    fn write_args(&self, es: &[ExprRef], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = Ok(());
        es.iter().enumerate().for_each(|(i, e)| {
            if i != 0 {
                let _ = write!(f, "{}", Self::DELIMITER);
            }
            res = res.and(write!(f, "{}", self.stringify(e)));
        });
        res
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{}", n),
            Literal::FloatParameter(_) => todo!(),
            Literal::StringParameter(_) => todo!(),
            Literal::String(_) => todo!(),
        }
    }
}

impl<'a> Display for Stringifier<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (0..self.level).into_iter().for_each(|_| {
            let _ = write!(f, "{}", Self::INDENT);
        });

        let expr = self.ctx.get_expr(self.e.clone()).ok_or(std::fmt::Error)?;
        match expr {
            Expr::Nop => write!(f, "nop"),
            Expr::Literal(l) => write!(f, "{}", l),
            Expr::Array(_) => todo!(),
            Expr::Var(s) => write!(f, "{}", self.get_symbol_str(s)),
            Expr::Let(id, body, then) => {
                self.write_indent(f);
                let _ = write!(
                    f,
                    "let {} = {}\n",
                    self.get_symbol_str(id),
                    self.stringify(body),
                );
                self.write_indent(f);
                write!(f, "{}", self.stringify(then))
            }
            Expr::BinOp(op, lhs, rhs) => {
                let lhs = self.stringify(lhs);
                let rhs = self.stringify(rhs);
                write!(f, "{lhs} {op} {rhs}")
            }
            Expr::App(callee, args) => {
                let callee = self.stringify(callee);
                write!(f, "{callee}(")
                    .and(self.write_args(args, f))
                    .and(write!(f, ")"))
            }
            Expr::AppExt(_, _) => todo!(),
            Expr::Lambda(ids, body) => {
                let args = self.join_symbols(ids.iter());
                let body = self.stringify(body);
                write!(f, "|{args}| {body}")
            }
            Expr::Track(_) => todo!(),
            Expr::Region(_, _, _) => todo!(),
            Expr::Project(_, _) => todo!(),
            Expr::Paren(e) => {
                let e = self.stringify(e);
                write!(f, "({e})")
            }
        }
    }
}
