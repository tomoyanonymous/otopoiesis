mod tokens;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use string_interner::StringInterner;
use tokens::{Op, Token};
mod error;
mod lexer;
pub mod stringifier;
use crate::error::ReportableError;
use crate::metadata::*;
use crate::Symbol;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::compiler::Context;
use crate::expr::{Expr, ExprRef, Literal};
use id_arena::{Arena, Id};
#[derive(Default)]
pub struct ParseContext {
    pub expr_storage: Arena<Expr>,
    pub span_storage: BTreeMap<Id<Expr>, Span>,
    pub interner: StringInterner,
}
impl ParseContext {
    pub fn get_expr(&self, id: ExprRef) -> Option<&Expr> {
        self.expr_storage.get(id.0)
    }
}

#[derive(Clone)]
pub struct ParseContextRef(pub Rc<RefCell<ParseContext>>);

impl ParseContextRef {
    pub fn new(c: ParseContext) -> Self {
        Self(Rc::new(RefCell::new(c)))
    }
    pub fn make_span(&self, e: ExprRef, span: Span) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ctx.span_storage.insert(e.0, span);
        e
    }
    pub fn make_lvar(&self, id: &str) -> Symbol {
        let mut ctx = self.0.borrow_mut();
        ctx.interner.get_or_intern(id)
    }
    pub fn make_nop(&self) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::Nop))
    }
    pub fn make_literal(&self, l: Literal) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::Literal(l)))
    }
    pub fn make_var(&self, v: &str) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        let id = ctx.interner.get_or_intern(v);
        ExprRef(ctx.expr_storage.alloc(Expr::Var(id)))
    }
    pub fn make_let(&self, ident: Symbol, body: ExprRef, then: ExprRef) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::Let(ident, body, then)))
    }
    pub fn make_lambda(&self, ident: &[Symbol], body: ExprRef) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::Lambda(ident.to_vec(), body)))
    }
    pub fn make_apply(&self, callee: ExprRef, args: &[ExprRef]) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::App(callee, args.to_vec())))
    }
    // pub fn make_block(&mut self,)
}
fn lvar_parser(ctx: ParseContextRef) -> impl Parser<Token, Symbol, Error = Simple<Token>> + Clone {
    select! {
        Token::Ident(s) => ctx.make_lvar(&s)
    }
}
fn literal_parser(
    ctx: ParseContextRef,
) -> impl Parser<Token, ExprRef, Error = Simple<Token>> + Clone {
    let ctxref = ctx.clone();
    select! {
        Token::Ident(v)=> ctxref.make_var(&v),
        Token::Int(x) => ctxref.make_literal(Literal::Number(x as f64)),
        Token::Float(x) =>ctxref.make_literal(Literal::Number(x.parse().unwrap())),
        Token::Str(s) => ctxref.make_literal(Literal::String(s)),
        // Token::SelfLit => Expr::Literal(Literal::SelfLit),
        // Token::Now => Expr::Literal(Literal::Now),
    }
    .map_with_span(move |e, s| {
        ctx.make_span(e.clone(), s);
        e
    })
    .labelled("value")
}
struct BinopParser(ParseContextRef);
impl BinopParser {
    pub fn exec(&self, x: ExprRef, y: ExprRef, op: Op, _opspan: Span) -> ExprRef {
        self.0.clone().make_apply(
            self.0.clone().make_var(op.get_associated_fn_name()),
            &[x.clone(), y.clone()],
        )
    }
}

fn expr_parser(ctx: ParseContextRef) -> impl Parser<Token, ExprRef, Error = Simple<Token>> + Clone {
    let ctxref = ctx.clone();

    let expr = recursive(|expr| {
        let lvar = lvar_parser(ctx.clone());
        let val = literal_parser(ctx.clone());
        // let expr_group = recursive(|expr_group| {
        let parenexpr = expr
            .clone()
            .delimited_by(just(Token::ParenBegin), just(Token::ParenEnd))
            .labelled("paren_expr");
        let ctxref = ctx.clone();

        let let_e = just(Token::Let)
            .ignore_then(lvar.clone())
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::LineBreak).or(just(Token::SemiColon)).repeated())
            .then(expr.clone().or_not())
            .map_with_span(move |((ident, body), then), _span| {
                let ctx = ctxref.clone();
                let then = match then {
                    Some(then) => then,
                    None => ctx.clone().make_nop(),
                };
                ctx.clone().make_let(ident, body, then)
            })
            .boxed()
            .labelled("let");
        let ctxref = ctx.clone();

        let lambda = lvar
            .clone()
            .separated_by(just(Token::Comma))
            .delimited_by(
                just(Token::LambdaArgBeginEnd),
                just(Token::LambdaArgBeginEnd),
            )
            .then(
                just(Token::Arrow)
                    // .ignore_then(type_parser())
                    .or_not(),
            )
            .then(expr.clone())
            .map_with_span(move |((ids, _type), body), _span| {
                ctxref.clone().make_lambda(&ids, body)
            })
            .labelled("lambda");

        // let macro_expand = select! { Token::MacroExpand(s) => Expr::Var(s,None) }
        //     .map_with_span(|e, s| WithMeta(e, s))
        //     .then_ignore(just(Token::ParenBegin))
        //     .then(expr_group.clone())
        //     .then_ignore(just(Token::ParenEnd))
        //     .map_with_span(|(id, then), s| {
        //         Expr::Escape(Box::new(WithMeta(
        //             Expr::Apply(Box::new(id), vec![then]),
        //             s.clone(),
        //         )))
        //     })
        //     .labelled("macroexpand");

        let atom = val
            .or(lambda)
            // .or(macro_expand)
            .or(let_e)
            // .map_with_span(move|e, s| ctxref.make_span(e, s))
            .or(parenexpr)
            .boxed()
            .labelled("atoms");
        let ctxref = ctx.clone();

        let items = expr
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>();

        let parenitems = items
            .clone()
            .delimited_by(just(Token::ParenBegin), just(Token::ParenEnd))
            .repeated();
        // let folder = |f, args| ctx.clone().borrow_mut().make_apply(f, args);
        let apply = atom
            .then(parenitems)
            .foldl(move |f, args: Vec<ExprRef>| ctxref.clone().make_apply(f, &args))
            .labelled("apply");

        let optoken = move |o: Op| {
            just(Token::Op(o)).map_with_span(|e, s| {
                (
                    match e {
                        Token::Op(o) => o,
                        _ => Op::Unknown(String::from("invalid")),
                    },
                    s,
                )
            })
        };

        let op = optoken(Op::Exponent);
        let ctxref = ctx.clone();
        let exponent = apply
            .clone()
            .then(op.then(apply).repeated())
            .foldl(move |x, ((op, opspan), y)| BinopParser(ctxref.clone()).exec(x, y, op, opspan))
            .boxed();

        let op = choice((
            optoken(Op::Product),
            optoken(Op::Divide),
            optoken(Op::Modulo),
        ));
        let ctxref = ctx.clone();
        let product = exponent
            .clone()
            .then(op.then(exponent).repeated())
            .foldl(move |x, ((op, opspan), y)| BinopParser(ctxref.clone()).exec(x, y, op, opspan))
            .boxed();
        let op = optoken(Op::Sum).or(optoken(Op::Minus));
        let ctxref = ctx.clone();

        let add = product
            .clone()
            .then(op.then(product).repeated())
            .foldl(move |x, ((op, opspan), y)| BinopParser(ctxref.clone()).exec(x, y, op, opspan))
            .boxed();

        let op = optoken(Op::Equal).or(optoken(Op::NotEqual));
        let ctxref = ctx.clone();

        let cmp = add
            .clone()
            .then(op.then(add).repeated())
            .foldl(move |x, ((op, opspan), y)| BinopParser(ctxref.clone()).exec(x, y, op, opspan))
            .boxed();
        let op = optoken(Op::And);
        let ctxref = ctx.clone();

        let cmp = cmp
            .clone()
            .then(op.then(cmp).repeated())
            .foldl(move |x, ((op, opspan), y)| BinopParser(ctxref.clone()).exec(x, y, op, opspan))
            .boxed();
        let op = optoken(Op::Or);
        let ctxref = ctx.clone();

        let cmp = cmp
            .clone()
            .then(op.then(cmp).repeated())
            .foldl(move |x, ((op, opspan), y)| BinopParser(ctxref.clone()).exec(x, y, op, opspan))
            .boxed();
        let op = choice((
            optoken(Op::LessThan),
            optoken(Op::LessEqual),
            optoken(Op::GreaterThan),
            optoken(Op::GreaterEqual),
        ));
        let ctxref = ctx.clone();
        let cmp = cmp
            .clone()
            .then(op.then(cmp).repeated())
            .foldl(move |x, ((op, opspan), y)| BinopParser(ctxref.clone()).exec(x, y, op, opspan))
            .boxed();
        let op = optoken(Op::Pipe);

        let pipe = cmp
            .clone()
            .then(op.then(cmp).repeated())
            .foldl(move |lhs, ((_, _), rhs)| {
                // let span = lhs.1.start..rhs.1.end;
                ctx.clone().make_apply(rhs, &[lhs])
            })
            .boxed();

        pipe
    });
    // expr_group contains let statement, assignment statement, function definiton,... they cannot be placed as an argument for apply directly.

    // let block = expr_group
    //     .clone()
    //     .padded_by(just(Token::LineBreak).or_not())
    //     .delimited_by(just(Token::BlockBegin), just(Token::BlockEnd))
    //     .map(|e: WithMeta<Expr>| Expr::Block(Some(Box::new(e))));

    // //todo: should be recursive(to paranthes be not needed)
    // let if_ = just(Token::If)
    //     .ignore_then(
    //         expr_group
    //             .clone()
    //             .delimited_by(just(Token::ParenBegin), just(Token::ParenEnd)),
    //     )
    //     .then(expr_group.clone())
    //     .then(
    //         just(Token::Else)
    //             .ignore_then(expr_group.clone().map(|e| Box::new(e)))
    //             .or_not(),
    //     )
    //     .map_with_span(|((cond, then), opt_else), s| {
    //         WithMeta(Expr::If(cond.into(), then.into(), opt_else), s)
    //     })
    //     .labelled("if");

    // block
    //     .map_with_span(|e, s| WithMeta(e, s))
    //     .or(if_)
    //     .or(expr.clone())
    // });
    let ctxref = ctxref.clone();
    expr.map_with_span(move |e, s| ctxref.make_span(e, s))
}
fn parser(ctx: ParseContextRef) -> impl Parser<Token, ExprRef, Error = Simple<Token>> + Clone {
    expr_parser(ctx)
}

pub fn parse(src: &str, ctx: ParseContextRef) -> Result<ExprRef, Vec<Box<dyn ReportableError>>> {
    let len = src.chars().count();
    let mut errs = Vec::<Box<dyn ReportableError>>::new();

    let (tokens, lex_errs) = lexer::lexer().parse_recovery(src);
    lex_errs
        .iter()
        .for_each(|e| errs.push(Box::new(error::ParseError::<char>(e.clone()))));

    if let Some(t) = tokens {
        let (ast, parse_errs) =
            parser(ctx).parse_recovery(chumsky::Stream::from_iter(len..len + 1, t.into_iter()));
        ast.ok_or_else(|| {
            parse_errs
                .iter()
                .for_each(|e| errs.push(Box::new(error::ParseError::<Token>(e.clone()))));
            errs
        })
    } else {
        Err(errs)
    }
}
