//! Parser for expressions

use winnow::ascii::{dec_int, float};
use winnow::combinator::{
    Infix, Postfix, Prefix, alt, cut_err, dispatch, expression, fail, opt, peek, preceded, repeat,
    terminated,
};
use winnow::stream::Stream;
use winnow::token::take_while;
use winnow::{ModalResult, Parser};

use crate::ir::Number;
use crate::ir::ast::op;
use crate::ir::ast::{self, Spanned};
use crate::parser::error::CtxErrBuilder;
use crate::parser::function::p_lambda;
use crate::parser::keyword::Ident;
use crate::parser::{
    Input, ParserExt, comma_list, delim, delim_cut, exp_string, keyword as k, p_ws, spanned,
};

// --- Primitives ---

/// Parses a number literal.
pub fn p_number<'a>(input: &mut Input<'a>) -> ModalResult<Number> {
    alt((
        // Integer if followed by range syntax
        //   otherwise float will take '.' off range operator
        terminated(dec_int, peek("..")).map(|n: i64| n as f64),
        float,
    ))
    .label("number")
    .ws()
    .parse_next(input)
}

/// Parses a string literal.
pub fn p_string<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    let start = input.checkpoint();
    '"'.parse_next(input)?;
    let s = take_while(0.., |c: char| c != '"').parse_next(input)?;
    let closing: ModalResult<char> = '"'.parse_next(input);
    if closing.is_err() {
        return Err(CtxErrBuilder::from_checkpoint(input, &start)
            .label("string")
            .expected("closing `\"`")
            .err);
    }
    p_ws.parse_next(input)?;
    Ok(s)
}

/// Parses a bool literal.
pub fn p_bool<'a>(input: &mut Input<'a>) -> ModalResult<bool> {
    alt((k::pk_true.map(|_| true), k::pk_false.map(|_| false)))
        .label("bool")
        .ws()
        .parse_next(input)
}

/// Parses a list literal.
pub fn p_list<'a>(input: &mut Input<'a>) -> ModalResult<ast::ListLiteral> {
    alt((
        k::pk_nil.map(|_| ast::ListLiteral::List(vec![])),
        delim_cut(
            '[',
            alt((
                (
                    p_expr.mws(),
                    opt(preceded(','.mws(), cut_err(p_expr).mws())),
                    preceded("..".mws(), cut_err(p_expr)),
                )
                    .map(|(start, second, end)| ast::ListLiteral::Range {
                        start: Box::new(start),
                        second: second.map(|s| Box::new(s)),
                        end: Box::new(end),
                    }),
                comma_list(0.., p_expr).map(|es| ast::ListLiteral::List(es)),
            )),
            ']',
        ),
    ))
    .label("list")
    .expected("comma-separated list")
    .expected("range `[x..y]`")
    .expected("range `[x,y..z]`")
    .ws()
    .parse_next(input)
}

/// Parses a literal expression.
pub fn p_literal<'a>(input: &mut Input<'a>) -> ModalResult<ast::Literal> {
    alt((
        p_number.map(ast::Literal::Number),
        p_string.map(|s| ast::Literal::String(s.to_string())),
        p_bool.map(ast::Literal::Bool),
        p_list.map(ast::Literal::List),
    ))
    .parse_next(input)
}

/// Parses a match arm: `<pattern> [if <guard>] => <body>`.
pub fn p_match_arm<'a>(input: &mut Input<'a>) -> ModalResult<ast::MatchArm> {
    (
        p_expr,
        opt(preceded(k::pk_if.ws(), p_expr)),
        preceded(exp_string("=>").ws(), p_expr),
    )
        .map(|(pattern, guard, body)| ast::MatchArm {
            pattern,
            guard,
            body,
        })
        .label("match arm")
        .parse_next(input)
}

/// Parses a match expression: `match <expr> { <arm>, <arm>, ... }`.
pub fn p_match<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    preceded(
        k::pk_match.ws(),
        cut_err((p_expr, delim('{', comma_list(1.., p_match_arm), '}'))),
    )
    .ws()
    .map(
        |(scrutinee, arms): (ast::SpanExpr, Vec<ast::MatchArm>)| ast::Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        },
    )
    .parse_next(input)
}

/// Parses an if expression: `if <cond> { <then> } else { <else> }`.
pub fn p_if<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    preceded(
        k::pk_if.ws(),
        cut_err((
            p_expr,
            delim('{', p_expr, '}').ws(),
            preceded(k::pk_else.ws(), delim('{', p_expr, '}')),
        )),
    )
    .ws()
    .map(
        |(condition, then_branch, else_branch): (ast::SpanExpr, ast::SpanExpr, ast::SpanExpr)| {
            ast::Expr::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            }
        },
    )
    .parse_next(input)
}

/// Parses a classified identifier as an atom expression.
///   User identifiers may have an optional `.field` suffix.
fn p_ident_atom<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    let (id, id_span) = k::p_ident.with_span().parse_next(input)?;
    let expr = match id {
        Ident::Std(s) => ast::Expr::Std(s),
        Ident::Const(c) => ast::Expr::Const(c),
        Ident::User(name) => {
            let fields: Vec<(String, std::ops::Range<usize>)> =
                repeat(0.., preceded('.', k::p_user_ident).with_span()).parse_next(input)?;
            let mut acc = Spanned {
                node: ast::Expr::Variable(name),
                span: Some(id_span),
            };
            for (field, field_span) in fields {
                let new_span = merge_spans(&acc.span, &Some(field_span));
                acc = Spanned {
                    node: ast::Expr::Field {
                        object: Box::new(acc),
                        field,
                    },
                    span: new_span,
                };
            }
            p_ws.parse_next(input)?;
            return Ok(acc.node);
        }
    };
    p_ws.parse_next(input)?;
    Ok(expr)
}

fn p_tuple_or_bracketed<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    delim('(', comma_list(1.., p_expr), ')')
        .map(|mut es: Vec<ast::SpanExpr>| {
            if es.len() == 1 {
                es.pop().unwrap().node
            } else {
                ast::Expr::Literal(ast::Literal::Tuple(es))
            }
        })
        .ws()
        .parse_next(input)
}

/// Parses an atom (returns a `Spanned<Expr>`).
pub fn p_atom<'a>(input: &mut Input<'a>) -> ModalResult<ast::SpanExpr> {
    spanned(alt((
        p_lambda.map(ast::Expr::Function),
        p_tuple_or_bracketed,
        p_literal.map(ast::Expr::Literal),
        p_if,
        p_match,
        p_ident_atom,
    )))
    .parse_next(input)
}

/// Parses an expression.
pub fn p_expr<'a>(input: &mut Input<'a>) -> ModalResult<ast::SpanExpr> {
    expression(p_atom).infix(dispatch! {alt((
        alt((">>", "&&", "||")),
        alt(("==", "!=", ">=", "<=", "//")),
        alt(("+", "-", "*", "/", "%", "^", ">", "<", ":")),
    )).mws();
        ">>" => Infix::Left(0, |_, arg: ast::SpanExpr, func: ast::SpanExpr| {
            let span = merge_spans(&arg.span, &func.span);
            Ok(Spanned { node: ast::Expr::Call { function: Box::new(func), args: vec![arg] }, span })
        }),
        "||" => Infix::Left(1, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Logic(op::Logic::Or))),
        "&&" => Infix::Left(2, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Logic(op::Logic::And))),
        // Comparisons
        "==" => Infix::Left(3, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Compare(op::Compare::Eq))),
        "!=" => Infix::Left(3, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Compare(op::Compare::Neq))),
        ">=" => Infix::Left(3, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Compare(op::Compare::Gte))),
        "<=" => Infix::Left(3, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Compare(op::Compare::Lte))),
        ">" => Infix::Left(3, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Compare(op::Compare::Gt))),
        "<" => Infix::Left(3, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Compare(op::Compare::Lt))),
        // Cons
        ":" => Infix::Right(4, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Cons)),
        // Arithmetic
        "+" => Infix::Left(5, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Arithmetic(op::Arithmetic::Add))),
        "-" => Infix::Left(5, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Arithmetic(op::Arithmetic::Sub))),
        "*" => Infix::Left(7, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Arithmetic(op::Arithmetic::Mul))),
        "/" => Infix::Left(7, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Arithmetic(op::Arithmetic::Div))),
        "//" => Infix::Left(7, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Arithmetic(op::Arithmetic::IntDiv))),
        "%" => Infix::Left(7, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Arithmetic(op::Arithmetic::Rem))),
        "^" => Infix::Right(10, |_, a: ast::SpanExpr, b: ast::SpanExpr| bin(a, b, op::Binary::Arithmetic(op::Arithmetic::Pow))),
        _ => fail,
    })
    .prefix(dispatch! {alt(("!", "-")).ws();
        "!" => Prefix(11, |_, a: ast::SpanExpr| {
            let span = a.span.clone();
            Ok(Spanned { node: ast::Expr::Unary { operator: op::Unary::Not, operand: Box::new(a) }, span })
        }),
        "-" => Prefix(9, |_, a: ast::SpanExpr| {
            let span = a.span.clone();
            Ok(Spanned { node: ast::Expr::Unary { operator: op::Unary::Neg, operand: Box::new(a) }, span })
        }),
        _ => fail,
    })
    .postfix(dispatch! { peek("(");
        "(" => Postfix(12, |input: &mut Input<'a>, e: ast::SpanExpr| {
            let (args, args_span): (Vec<ast::SpanExpr>, _) = delim('(',comma_list(0.., p_expr),')')
                    .ws()
                    .with_span()
                    .parse_next(input)?;
            let span = merge_spans(&e.span, &Some(args_span));
            Ok(Spanned {
                node: ast::Expr::Call { function: Box::new(e), args },
                span,
            })
        }),
        _ => fail,
    })
    .parse_next(input)
}

/// Build a binary Spanned expression from two spanned operands.
fn bin(a: ast::SpanExpr, b: ast::SpanExpr, operator: op::Binary) -> ModalResult<ast::SpanExpr> {
    let span = merge_spans(&a.span, &b.span);
    Ok(Spanned {
        node: ast::Expr::Binary {
            operator,
            left: Box::new(a),
            right: Box::new(b),
        },
        span,
    })
}

/// Combine two optional spans into one covering both.
fn merge_spans(a: &Option<ast::Span>, b: &Option<ast::Span>) -> Option<ast::Span> {
    Some(a.as_ref()?.start..b.as_ref()?.end)
}
