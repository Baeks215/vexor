//! Parser for graphic components

use crate::ir::ast;
use crate::parser::expr::p_expr;
use crate::parser::keyword::{pk_circle, pk_rect, pk_text};
use crate::parser::{Input, bracketed, lexeme};
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
    preceded(
        pk_rect,
        bracketed(separated_pair(p_expr, lexeme(','), p_expr)),
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pg_circle() {
        let mut input = Input::new("circle(10)");
        let res = pg_circle.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Graphic::Circle {
                radius: Box::new(ast::Expr::LNumber(10.0))
            }
        );
        assert_eq!(*input, "");

        let mut input = Input::new("circle(5 + 5)");
        let res = pg_circle.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Graphic::Circle {
                radius: Box::new(ast::Expr::Binary {
                    operator: ast::OpBin::Add,
                    left: Box::new(ast::Expr::LNumber(5.0)),
                    right: Box::new(ast::Expr::LNumber(5.0))
                })
            }
        );
    }

    #[test]
    fn test_pg_rect() {
        let mut input = Input::new("rect(10, 20)");
        let res = pg_rect.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Graphic::Rect {
                width: Box::new(ast::Expr::LNumber(10.0)),
                height: Box::new(ast::Expr::LNumber(20.0))
            }
        );
        assert_eq!(*input, "");
    }

    #[test]
    fn test_pg_text() {
        let mut input = Input::new("text(\"hello\")");
        let res = pg_text.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Graphic::Text(Box::new(ast::Expr::LString("hello".to_string())))
        );
        assert_eq!(*input, "");
    }

    #[test]
    fn test_p_graphic() {
        let mut input = Input::new("circle(10)  ");
        assert!(p_graphic.parse_next(&mut input).is_ok());
        assert_eq!(*input, "");

        let mut input = Input::new("rect(1, 1) a");
        assert!(p_graphic.parse_next(&mut input).is_ok());
        assert_eq!(*input, "a");

        let mut input = Input::new("text(\"foo\") ");
        assert!(p_graphic.parse_next(&mut input).is_ok());
        assert_eq!(*input, "");
    }
}
