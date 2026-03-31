//! Parser for graphic components

use crate::ir::ast;
use crate::parser::common::keyword::{pk_circle, pk_rect, pk_text};
use crate::parser::common::{Input, bracketed, lexeme};
use crate::parser::expr::p_expr;
use winnow::combinator::{alt, preceded, separated_pair};
use winnow::{ModalResult, Parser};

fn pg_circle<'a>(input: &mut Input<'a>) -> ModalResult<ast::Graphic> {
    preceded(pk_circle, bracketed(p_expr))
        .map(|e| ast::Graphic::Circle {
            radius: Box::new(e),
        })
        .parse_next(input)
}

fn pg_rect<'a>(input: &mut Input<'a>) -> ModalResult<ast::Graphic> {
    preceded(pk_rect, bracketed(separated_pair(p_expr, ',', p_expr)))
        .map(|(w, h)| ast::Graphic::Rect {
            width: Box::new(w),
            height: Box::new(h),
        })
        .parse_next(input)
}

fn pg_text<'a>(input: &mut Input<'a>) -> ModalResult<ast::Graphic> {
    preceded(pk_text, bracketed(p_expr))
        .map(|e| ast::Graphic::Text(Box::new(e)))
        .parse_next(input)
}

/// Parses a basic graphic.
pub fn p_graphic<'a>(input: &mut Input<'a>) -> ModalResult<ast::Graphic> {
    lexeme(alt((pg_circle, pg_rect, pg_text))).parse_next(input)
}
