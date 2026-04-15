//! Parser for expressions

use crate::ir::Number;
use crate::ir::ast;
use crate::parser::graphic::p_graphic;
use crate::parser::keyword::pk_color;
use crate::parser::p_identifier_no_ws;
use crate::parser::{Input, bracketed, lexeme, p_identifier};
use winnow::ascii::float;
use winnow::combinator::{Infix, alt, delimited, dispatch, expression, fail, preceded, separated};
use winnow::error::StrContext;
use winnow::token::{any, take_while};
use winnow::{ModalResult, Parser};

// --- Primitives ---

/// Parses a number literal.
pub fn p_number<'a>(input: &mut Input<'a>) -> ModalResult<Number> {
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
            separated(4, p_expr, lexeme(',')).map(|mut es: Vec<ast::Expr>| ast::Color::Rgba {
                r: Box::new(es.remove(0)),
                g: Box::new(es.remove(0)),
                b: Box::new(es.remove(0)),
                a: Box::new(es.remove(0)),
            }),
        ),
    ))
    .parse_next(input)
}

/// Parses a function call.
pub fn p_call<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    lexeme((
        p_identifier_no_ws,
        bracketed(separated(0.., p_expr, lexeme(','))),
    ))
    .map(|(function, args)| ast::Expr::Call {
        function: function.to_string(),
        args,
    })
    .parse_next(input)
}

/// Parses an atom.
pub fn p_atom<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    alt((
        p_number.map(|n| ast::Expr::LNumber(n)),
        p_string.map(|s| ast::Expr::LString(s.to_string())),
        p_color.map(|c| ast::Expr::LColor(c)),
        p_graphic.map(|g| ast::Expr::LGraphic(g)),
        p_call,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p_number() {
        let mut input = Input::new("123.45  ");
        assert_eq!(p_number.parse_next(&mut input).unwrap(), 123.45);
        assert_eq!(*input, "");

        let mut input = Input::new("42 ");
        assert_eq!(p_number.parse_next(&mut input).unwrap(), 42.0);
        assert_eq!(*input, "");

        let mut input = Input::new("-10.5 ");
        assert_eq!(p_number.parse_next(&mut input).unwrap(), -10.5);
        assert_eq!(*input, "");

        let mut input = Input::new("-100 ");
        assert_eq!(p_number.parse_next(&mut input).unwrap(), -100.0);
        assert_eq!(*input, "");

        // Invalid for disconnected negative
        let mut input = Input::new("- 100 ");
        assert!(p_number.parse_next(&mut input).is_err());
    }

    #[test]
    fn test_p_string() {
        let mut input = Input::new("\"hello world\"  ");
        assert_eq!(p_string.parse_next(&mut input).unwrap(), "hello world");
        assert_eq!(*input, "");
    }

    #[test]
    fn test_p_color() {
        let mut input = Input::new("color.rgb(0.5, 0.6, 0.1, 1)");
        let res = p_color.parse_next(&mut input).unwrap();
        match res {
            ast::Color::Rgba { r, g, b, a } => {
                assert_eq!(*r, ast::Expr::LNumber(0.5));
                assert_eq!(*g, ast::Expr::LNumber(0.6));
                assert_eq!(*b, ast::Expr::LNumber(0.1));
                assert_eq!(*a, ast::Expr::LNumber(1.0));
            }
        }
    }

    #[test]
    fn test_p_call() {
        let mut input = Input::new("foo(1, 2 + 3)");
        let res = p_call.parse_next(&mut input).unwrap();
        match res {
            ast::Expr::Call { function, args } => {
                assert_eq!(function, "foo");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], ast::Expr::LNumber(1.0));
                assert_eq!(
                    args[1],
                    ast::Expr::Binary {
                        operator: ast::OpBin::Add,
                        left: Box::new(ast::Expr::LNumber(2.0)),
                        right: Box::new(ast::Expr::LNumber(3.0)),
                    }
                );
            }
            _ => panic!("Expected Call, got {:?}", res),
        }

        // Zero-arg call
        let mut input = Input::new("bar()");
        let res = p_call.parse_next(&mut input).unwrap();
        match res {
            ast::Expr::Call { function, args } => {
                assert_eq!(function, "bar");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Call, got {:?}", res),
        }
    }

    #[test]
    fn test_p_expr() {
        // 1 + 2 * 3  => 1 + (2 * 3)
        let mut input = Input::new("1 + 2 * 3");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Add,
                left: Box::new(ast::Expr::LNumber(1.0)),
                right: Box::new(ast::Expr::Binary {
                    operator: ast::OpBin::Mul,
                    left: Box::new(ast::Expr::LNumber(2.0)),
                    right: Box::new(ast::Expr::LNumber(3.0)),
                })
            }
        );

        // 1 * 2 + 3 => (1 * 2) + 3
        let mut input = Input::new("1 * 2 + 3");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Add,
                left: Box::new(ast::Expr::Binary {
                    operator: ast::OpBin::Mul,
                    left: Box::new(ast::Expr::LNumber(1.0)),
                    right: Box::new(ast::Expr::LNumber(2.0)),
                }),
                right: Box::new(ast::Expr::LNumber(3.0)),
            }
        );

        // 1 - 2 - 3 => (1 - 2) - 3
        let mut input = Input::new("1 - 2 - 3");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Sub,
                left: Box::new(ast::Expr::Binary {
                    operator: ast::OpBin::Sub,
                    left: Box::new(ast::Expr::LNumber(1.0)),
                    right: Box::new(ast::Expr::LNumber(2.0)),
                }),
                right: Box::new(ast::Expr::LNumber(3.0)),
            }
        );

        // 1 + -2
        let mut input = Input::new("1 + -2");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Add,
                left: Box::new(ast::Expr::LNumber(1.0)),
                right: Box::new(ast::Expr::LNumber(-2.0)),
            }
        );

        // Whitespace
        let mut input = Input::new("1   +  2 *  3  ");
        assert!(p_expr.parse_next(&mut input).is_ok());
        assert_eq!(*input, "");
    }
}
