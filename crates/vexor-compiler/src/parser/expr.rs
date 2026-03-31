//! Parser for expressions

use crate::ir::ast;
use crate::parser::common::keyword::pk_color;
use crate::parser::common::{Input, bracketed, lexeme};
use winnow::ascii::float;
use winnow::combinator::{alt, delimited, dispatch, preceded, repeat, separated};
use winnow::error::StrContext;
use winnow::token::{take_until, take_while};
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
                g: Box::new(es.remove(1)),
                b: Box::new(es.remove(2)),
                a: Box::new(es.remove(3)),
            }),
        ),
    ))
    .parse_next(input)
}

/// Parses an expression.
pub fn p_expr<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    todo!()
}
