mod tokens;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use tokens::{Op, Token};
mod error;
mod lexer;
pub mod stringifier;
use crate::error::ReportableError;
use crate::expr::Attribute;
use crate::metadata::*;
use crate::Symbol;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::expr::{Expr, ExprRef, Literal};
use crate::Interner;
use id_arena::{Arena, Id};

pub struct ParseContext {
    pub expr_storage: Arena<Expr>,
    pub span_storage: BTreeMap<Id<Expr>, Span>,
    pub interner: Interner,
}
impl Default for ParseContext {
    fn default() -> Self {
        Self {
            expr_storage: Default::default(),
            span_storage: Default::default(),
            interner: Interner::new(),
        }
    }
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
        Symbol(ctx.interner.get_or_intern(id))
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
        let id = Symbol(ctx.interner.get_or_intern(v));
        ExprRef(ctx.expr_storage.alloc(Expr::Var(id)))
    }
    pub fn make_let(&self, ident: Symbol, body: ExprRef, then: ExprRef) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::Let(ident, body, then)))
    }
    pub fn make_with_attribute(&self, attr: Attribute, body: ExprRef) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::WithAttribute(attr, body)))
    }
    pub fn make_lambda(&self, ident: &[Symbol], body: ExprRef) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::Lambda(ident.to_vec(), body)))
    }
    pub fn make_apply(&self, callee: ExprRef, args: &[ExprRef]) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::App(callee, args.to_vec())))
    }
    pub fn make_binop(&self, op: Op, lhs: ExprRef, rhs: ExprRef) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::BinOp(op, lhs, rhs)))
    }
    pub fn make_paren(&self, body: ExprRef) -> ExprRef {
        let mut ctx = self.0.borrow_mut();
        ExprRef(ctx.expr_storage.alloc(Expr::Paren(body)))
    }
    // pub fn make_block(&mut self,)
}
fn lvar_parser(ctx: ParseContextRef) -> impl Parser<Token, Symbol, Error = Simple<Token>> + Clone {
    select! {
        Token::Ident(s) => ctx.make_lvar(&s)
    }
}

fn binop_parser<P, O, F>(
    atom: P,
    op: O,
    folder: &F,
) -> impl Parser<Token, ExprRef, Error = Simple<Token>> + Clone
where
    P: Parser<Token, ExprRef, Error = Simple<Token>> + Clone,
    O: Parser<Token, Op, Error = Simple<Token>> + Clone,
    F: Fn(ExprRef, (Op, ExprRef)) -> ExprRef + Clone,
{
    atom.clone()
        .then(op.then(atom).repeated())
        .foldl(folder.clone())
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
    pub fn exec(&self, x: ExprRef, y: ExprRef, op: Op) -> ExprRef {
        self.0.clone().make_binop(op, x, y)
    }
}
fn number_parser() -> impl Parser<Token, f64, Error = Simple<Token>> + Clone {
    select! {
        Token::Float(s)=> s.parse::<f64>().unwrap(),
        Token::Int(i) => i as f64
    }
}
fn range_parser() -> impl Parser<Token, (f64, f64), Error = Simple<Token>> + Clone {
    number_parser()
        .then_ignore(just(Token::DoubleDot))
        .then(number_parser())
}
fn attribute_parser(
    ctx: ParseContextRef,
) -> impl Parser<Token, Attribute, Error = Simple<Token>> + Clone {
    lvar_parser(ctx)
        .then(range_parser().delimited_by(just(Token::ParenBegin), just(Token::ParenEnd)))
        .delimited_by(
            just(Token::SharpAngleBracketBegin),
            just(Token::AngleBracketEnd),
        )
        .map(|(symbol, (start, end))| Attribute(symbol, start..=end))
}
fn expr_parser(ctx: ParseContextRef) -> impl Parser<Token, ExprRef, Error = Simple<Token>> + Clone {
    let ctxref = ctx.clone();

    let expr = recursive(|expr| {
        let lvar = lvar_parser(ctx.clone());
        let val = literal_parser(ctx.clone());
        // let expr_group = recursive(|expr_group| {
        let ctxref = ctx.clone();
        let parenexpr = expr
            .clone()
            .delimited_by(just(Token::ParenBegin), just(Token::ParenEnd))
            .map(move |e| ctxref.make_paren(e))
            .labelled("paren_expr");
        let ctxref = ctx.clone();

        let let_e = attribute_parser(ctxref.clone())
            .then_ignore(just(Token::LineBreak))
            .or_not()
            .then_ignore(just(Token::Let))
            .then(lvar.clone())
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::LineBreak).or(just(Token::SemiColon)).repeated())
            .then(expr.clone().or_not())
            .map_with_span(move |(((attr, ident), body), then), _span| {
                let ctx = ctxref.clone();
                let then = match then {
                    Some(then) => then,
                    None => ctx.clone().make_nop(),
                };
                let res = ctx.clone().make_let(ident, body, then);
                if let Some(a) = attr {
                    ctx.make_with_attribute(a, res)
                } else {
                    res
                }
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
            just(Token::Op(o))
                .then_ignore(just(Token::LineBreak).repeated())
                .map(|tk| match tk {
                    Token::Op(o) => o,
                    _ => panic!(),
                })
        };
        let ctxref = ctx.clone();
        let folder = move |x, (o, y)| BinopParser(ctxref.clone()).exec(x, y, o);
        let exponent = {
            let op = optoken(Op::Exponent);
            binop_parser(apply, op, &folder)
        };
        let product = {
            let op = choice((
                optoken(Op::Product),
                optoken(Op::Divide),
                optoken(Op::Modulo),
            ));
            binop_parser(exponent, op, &folder)
        };

        let add = {
            let op = optoken(Op::Sum).or(optoken(Op::Minus));
            binop_parser(product, op, &folder)
        };

        let cmp = {
            let op = optoken(Op::Equal).or(optoken(Op::NotEqual));
            binop_parser(add, op, &folder)
        };

        let cmp = {
            let op = optoken(Op::And);
            binop_parser(cmp, op, &folder)
        };
        let cmp = {
            let op = optoken(Op::Or);
            binop_parser(cmp, op, &folder)
        };
        let cmp = {
            let op = choice((
                optoken(Op::LessThan),
                optoken(Op::LessEqual),
                optoken(Op::GreaterThan),
                optoken(Op::GreaterEqual),
            ));
            binop_parser(cmp, op, &folder)
        };
        let op = optoken(Op::Pipe);

        let pipe = cmp
            .clone()
            .then(op.then(cmp).repeated())
            .foldl(move |lhs, (_, rhs)| {
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

#[cfg(test)]
mod test {
    #[test]
    fn attribute_parser_test() {
        use super::*;
        let src = "#[param(0.0..1.0)]";
        let len = src.chars().count();
        let ctx = ParseContextRef(Rc::new(RefCell::new(ParseContext::default())));
        let (tokens_opt, lex_errs) = super::lexer::lexer().parse_recovery(src);
        lex_errs.iter().for_each(|f| println!("{}", f.to_string()));
        assert!(lex_errs.is_empty());
        if let Some(t) = tokens_opt {
            let (attribute, errs) = attribute_parser(ctx)
                .parse_recovery(chumsky::Stream::from_iter(len..len + 1, t.into_iter()));
            if let Some(attr) = attribute {
                assert_eq!(attr.1, 0.0f64..=1.0f64);
            } else {
                errs.iter().for_each(|e| {
                    println!("{}", e.to_string());
                });
                panic!()
            }
        } else {
            panic!()
        }
    }
}
