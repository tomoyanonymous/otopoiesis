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
    const DELIMITER: &str = ",";
    const INDENT: &str = "  ";
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
            Expr::Nop => todo!(),
            Expr::Literal(l) => write!(f, "{}", l),
            Expr::Array(_) => todo!(),
            Expr::Var(s) => write!(f, "{}", self.get_symbol_str(s)),
            Expr::Let(id, body, then) => {
                write!(
                    f,
                    "let {} = {}\n{}",
                    self.get_symbol_str(id),
                    self.stringify(body),
                    self.stringify(then)
                )
            }
            Expr::App(callee, args) => write!(f, "{}", self.stringify(callee))
                .and(write!(f, "("))
                .and(self.write_args(args, f))
                .and(write!(f, ")")),
            Expr::AppExt(_, _) => todo!(),
            Expr::Lambda(_, _) => todo!(),
            Expr::Track(_) => todo!(),
            Expr::Region(_, _, _) => todo!(),
            Expr::Project(_, _) => todo!(),
        }
    }
}
