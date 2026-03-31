//! Parser for expressions

use crate::ir::ast;
use crate::parser::common::keyword::pk_color;
use crate::parser::common::{Input, bracketed, lexeme, p_identifier};
use crate::parser::graphic::p_graphic;
use winnow::ascii::float;
use winnow::combinator::{Infix, alt, delimited, dispatch, expression, fail, preceded, separated};
use winnow::error::StrContext;
use winnow::token::{any, take_while};
use winnow::{ModalResult, Parser};

// --- Primitives ---

/// Parses a number literal.
pub fn p_number<'a>(input: &mut Input<'a>) -> ModalResult<ast::Number> {
    lexeme(float.context(StrContext::Label("number"))).parse_next(input)
}

/// Parses a string literal.
pub fn p_string<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    lexeme(delimited('"', take_while(0.., |c: char| c != '"'), '"')).parse_next(input)
}

/// Parses a color.
pub fn p_color<'a>(input: &mut Input<'a>) -> ModalResult<ast::Color> {
    lexeme(preceded(
        (pk_color, ".rgb"),
        bracketed(
            separated(4, p_expr, ",").map(|mut es: Vec<ast::Expr>| ast::Color::Rgba {
                r: Box::new(es.remove(0)),
                g: Box::new(es.remove(0)),
                b: Box::new(es.remove(0)),
                a: Box::new(es.remove(0)),
            }),
        ),
    ))
    .parse_next(input)
}

/// Parses an atom.
pub fn p_atom<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    alt((
        p_number.map(|n| ast::Expr::LNumber(n)),
        p_string.map(|s| ast::Expr::LString(s.to_string())),
        p_color.map(|c| ast::Expr::LColor(c)),
        p_graphic.map(|g| ast::Expr::LGraphic(g)),
        p_identifier.map(|s| ast::Expr::Variable(s.to_string())),
    ))
    .parse_next(input)
}

/// Parses an expression.
pub fn p_expr<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    expression(p_atom).infix(dispatch! {lexeme(any);
        '+' => Infix::Left(5, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Add, left: Box::new(a), right: Box::new(b) })),
        '-' => Infix::Left(5, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Sub, left: Box::new(a), right: Box::new(b) })),
        '*' => Infix::Left(7, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Mul, left: Box::new(a), right: Box::new(b) })),
        '/' => Infix::Left(7, |_, a, b| Ok(ast::Expr::Binary { operator: ast::OpBin::Div, left: Box::new(a), right: Box::new(b) })),
        _ => fail,
    }).parse_next(input)
}
