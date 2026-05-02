//! Parser for expressions

use crate::ir::Number;
use crate::ir::ast;
use crate::parser::keyword::pk_rgb;
use crate::parser::keyword::{pk_else, pk_false, pk_if, pk_match, pk_true};
use crate::parser::object::p_object;
use crate::parser::p_identifier_no_ws;
use crate::parser::p_raw_identifier_no_ws;
use crate::parser::{Input, WhiteSpaceParser, braced, bracketed, p_identifier};
use winnow::ascii::float;
use winnow::combinator::{
    Infix, Prefix, alt, delimited, dispatch, expression, fail, opt, preceded, separated,
};
use winnow::error::StrContext;
use winnow::token::take_while;
use winnow::{ModalResult, Parser};

// --- Primitives ---

/// Parses a number literal.
pub fn p_number<'a>(input: &mut Input<'a>) -> ModalResult<Number> {
    float
        .context(StrContext::Label("number"))
        .ws()
        .parse_next(input)
}

/// Parses a string literal.
pub fn p_string<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    delimited('"', take_while(0.., |c: char| c != '"'), '"')
        .ws()
        .parse_next(input)
}

/// Parses a bool literal.
pub fn p_bool<'a>(input: &mut Input<'a>) -> ModalResult<bool> {
    alt((pk_true.map(|_| true), pk_false.map(|_| false)))
        .ws()
        .parse_next(input)
}

/// Parses a color.
pub fn p_color<'a>(input: &mut Input<'a>) -> ModalResult<ast::Color> {
    preceded(
        pk_rgb,
        bracketed(
            separated(4, p_expr, ','.ws()).map(|mut es: Vec<ast::Expr>| ast::Color::Rgba {
                r: Box::new(es.remove(0)),
                g: Box::new(es.remove(0)),
                b: Box::new(es.remove(0)),
                a: Box::new(es.remove(0)),
            }),
        ),
    )
    .ws()
    .parse_next(input)
}

/// Parses a function call.
pub fn p_call<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    (
        p_identifier_no_ws,
        bracketed(separated(0.., p_expr, ','.ws())),
    )
        .ws()
        .map(|(function, args)| ast::Expr::Call {
            function: function.to_string(),
            args,
        })
        .parse_next(input)
}

/// Parses a literal expression.
pub fn p_literal<'a>(input: &mut Input<'a>) -> ModalResult<ast::Literal> {
    alt((
        p_number.map(ast::Literal::Number),
        p_string.map(|s| ast::Literal::String(s.to_string())),
        p_bool.map(ast::Literal::Bool),
        p_color.map(ast::Literal::Color),
        p_object.map(ast::Literal::Object),
    ))
    .parse_next(input)
}

/// Parses a pattern: a literal or a binding identifier.
pub fn p_pattern<'a>(input: &mut Input<'a>) -> ModalResult<ast::Pattern> {
    alt((
        p_literal.map(ast::Pattern::Literal),
        p_identifier.map(|s| ast::Pattern::Binding(s.to_string())),
    ))
    .parse_next(input)
}

/// Parses a match arm: `<pattern> [if <guard>] => <body>`.
pub fn p_match_arm<'a>(input: &mut Input<'a>) -> ModalResult<ast::MatchArm> {
    (
        p_pattern,
        opt(preceded(pk_if.ws(), p_expr)),
        preceded("=>".ws(), p_expr),
    )
        .map(|(pattern, guard, body)| ast::MatchArm {
            pattern,
            guard,
            body,
        })
        .parse_next(input)
}

/// Parses a match expression: `match <expr> { <arm>, <arm>, ... }`.
pub fn p_match<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    preceded(
        pk_match.ws(),
        (p_expr, braced(separated(1.., p_match_arm, ",".mws()))),
    )
    .ws()
    .map(
        |(scrutinee, arms): (ast::Expr, Vec<ast::MatchArm>)| ast::Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        },
    )
    .parse_next(input)
}

/// Parses an if expression: `if <cond> { <then> } else { <else> }`.
pub fn p_if<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    (
        preceded(pk_if.ws(), p_expr),
        braced(p_expr).ws(),
        preceded(pk_else.ws(), braced(p_expr)),
    )
        .ws()
        .map(
            |(condition, then_branch, else_branch): (ast::Expr, ast::Expr, ast::Expr)| {
                ast::Expr::If {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                }
            },
        )
        .parse_next(input)
}

/// Parses identifier or object field access
pub fn p_identifier_or_field<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    (
        p_identifier_no_ws.map(str::to_string),
        opt(preceded(".", p_raw_identifier_no_ws.map(str::to_string))),
    )
        .ws()
        .map(|(var, field)| match field {
            Some(field) => ast::Expr::Field { object: var, field },
            None => ast::Expr::Variable(var),
        })
        .parse_next(input)
}

/// Parses an atom.
pub fn p_atom<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    alt((
        p_literal.map(ast::Expr::Literal),
        p_if,
        p_match,
        p_call,
        p_identifier_or_field,
    ))
    .parse_next(input)
}

/// Parses an expression.
pub fn p_expr<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    expression(p_atom).infix(dispatch! {alt((
        alt(("&&", "||")),
        alt(("==", "!=", ">=", "<=")),
        alt(("+", "-", "*", "/", ">", "<")),
    )).ws();
        "||" => Infix::Left(1, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Or, left: Box::new(a), right: Box::new(b) })),
        "&&" => Infix::Left(2, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::And, left: Box::new(a), right: Box::new(b) })),
        "==" => Infix::Left(3, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Eq, left: Box::new(a), right: Box::new(b) })),
        "!=" => Infix::Left(3, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Neq, left: Box::new(a), right: Box::new(b) })),
        ">=" => Infix::Left(4, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Gte, left: Box::new(a), right: Box::new(b) })),
        "<=" => Infix::Left(4, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Lte, left: Box::new(a), right: Box::new(b) })),
        ">" => Infix::Left(4, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Gt, left: Box::new(a), right: Box::new(b) })),
        "<" => Infix::Left(4, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Lt, left: Box::new(a), right: Box::new(b) })),
        "+" => Infix::Left(5, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Add, left: Box::new(a), right: Box::new(b) })),
        "-" => Infix::Left(5, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Sub, left: Box::new(a), right: Box::new(b) })),
        "*" => Infix::Left(7, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Mul, left: Box::new(a), right: Box::new(b) })),
        "/" => Infix::Left(7, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Div, left: Box::new(a), right: Box::new(b) })),
        _ => fail,
    })
    .prefix(dispatch! {"!";
        "!" => Prefix(11, |_, a| Ok(ast::Expr::Unary { operator: ast::OpUn::Not, operand: Box::new(a) })),
        _ => fail,
    })
    .parse_next(input)
}
