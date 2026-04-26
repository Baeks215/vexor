//! Parser for expressions

use crate::ir::Number;
use crate::ir::ast;
use crate::parser::keyword::{pk_color, pk_else, pk_false, pk_if, pk_match, pk_true};
use crate::parser::object::p_object;
use crate::parser::p_identifier_no_ws;
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
        (pk_color, ".rgb"),
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

/// Parses a pattern.
///   A bare identifier is a binding, any other expression is a literal match.
pub fn p_pattern<'a>(input: &mut Input<'a>) -> ModalResult<ast::Pattern> {
    p_expr
        .map(|e| match e {
            ast::Expr::Variable(name) => ast::Pattern::Binding(name),
            other => ast::Pattern::Literal(other),
        })
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
        (p_expr, braced(separated(1.., p_match_arm, ",".mws())).ws()),
    )
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
        preceded(pk_else.ws(), braced(p_expr).ws()),
    )
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

/// Parses an atom.
pub fn p_atom<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    alt((
        p_number.map(|n| ast::Expr::LNumber(n)),
        p_string.map(|s| ast::Expr::LString(s.to_string())),
        p_bool.map(|b| ast::Expr::LBool(b)),
        p_color.map(|c| ast::Expr::LColor(c)),
        p_object.map(|o| ast::Expr::LObject(o)),
        p_if,
        p_match,
        p_call,
        p_identifier.map(|s| ast::Expr::Variable(s.to_string())),
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
    .prefix(dispatch! {"!".ws();
        "!" => Prefix(11, |_, a| Ok(ast::Expr::Unary { operator: ast::OpUn::Not, operand: Box::new(a) })),
        _ => fail,
    })
    .parse_next(input)
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

    #[test]
    fn test_p_bool() {
        let mut input = Input::new("true");
        assert_eq!(p_bool.parse_next(&mut input).unwrap(), true);
        assert_eq!(*input, "");

        let mut input = Input::new("false  ");
        assert_eq!(p_bool.parse_next(&mut input).unwrap(), false);
        assert_eq!(*input, "");

        let mut input = Input::new("nope");
        assert!(p_bool.parse_next(&mut input).is_err());
    }

    #[test]
    fn test_p_atom_bool() {
        let mut input = Input::new("true");
        assert_eq!(
            p_atom.parse_next(&mut input).unwrap(),
            ast::Expr::LBool(true)
        );

        let mut input = Input::new("false");
        assert_eq!(
            p_atom.parse_next(&mut input).unwrap(),
            ast::Expr::LBool(false)
        );
    }

    #[test]
    fn test_p_expr_compare() {
        // -1 > -2
        let mut input = Input::new("-1 > -2");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Gt,
                left: Box::new(ast::Expr::LNumber(-1.0)),
                right: Box::new(ast::Expr::LNumber(-2.0)),
            }
        );

        // -1 >= 2
        let mut input = Input::new("-1 >= 2");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Gte,
                left: Box::new(ast::Expr::LNumber(-1.0)),
                right: Box::new(ast::Expr::LNumber(2.0)),
            }
        );

        // -1 == -1
        let mut input = Input::new("-1 == -1");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Eq,
                left: Box::new(ast::Expr::LNumber(-1.0)),
                right: Box::new(ast::Expr::LNumber(-1.0)),
            }
        );

        // 1 != -2
        let mut input = Input::new("1 != -2");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Neq,
                left: Box::new(ast::Expr::LNumber(1.0)),
                right: Box::new(ast::Expr::LNumber(-2.0)),
            }
        );

        // -1 <= 2
        let mut input = Input::new("-1 <= 2");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Lte,
                left: Box::new(ast::Expr::LNumber(-1.0)),
                right: Box::new(ast::Expr::LNumber(2.0)),
            }
        );

        // -3 < -2
        let mut input = Input::new("-3 < -2");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Lt,
                left: Box::new(ast::Expr::LNumber(-3.0)),
                right: Box::new(ast::Expr::LNumber(-2.0)),
            }
        );
    }

    #[test]
    fn test_p_expr_compare_precedence() {
        // 1 + 2 > 3 => (1 + 2) > 3
        let mut input = Input::new("1 + 2 > 3");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Gt,
                left: Box::new(ast::Expr::Binary {
                    operator: ast::OpBin::Add,
                    left: Box::new(ast::Expr::LNumber(1.0)),
                    right: Box::new(ast::Expr::LNumber(2.0)),
                }),
                right: Box::new(ast::Expr::LNumber(3.0)),
            }
        );

        // parser-only: 1 == 2 + 3 => 1 == (2 + 3)
        let mut input = Input::new("1 == 2 + 3");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Eq,
                left: Box::new(ast::Expr::LNumber(1.0)),
                right: Box::new(ast::Expr::Binary {
                    operator: ast::OpBin::Add,
                    left: Box::new(ast::Expr::LNumber(2.0)),
                    right: Box::new(ast::Expr::LNumber(3.0)),
                }),
            }
        );
    }

    #[test]
    fn test_p_expr_logical() {
        // true && false
        let mut input = Input::new("true && false");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::And,
                left: Box::new(ast::Expr::LBool(true)),
                right: Box::new(ast::Expr::LBool(false)),
            }
        );

        // true || false
        let mut input = Input::new("true || false");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Or,
                left: Box::new(ast::Expr::LBool(true)),
                right: Box::new(ast::Expr::LBool(false)),
            }
        );

        // !true
        let mut input = Input::new("!true");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Unary {
                operator: ast::OpUn::Not,
                operand: Box::new(ast::Expr::LBool(true)),
            }
        );

        // !!true
        let mut input = Input::new("!!true");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Unary {
                operator: ast::OpUn::Not,
                operand: Box::new(ast::Expr::Unary {
                    operator: ast::OpUn::Not,
                    operand: Box::new(ast::Expr::LBool(true)),
                }),
            }
        );
    }

    #[test]
    fn test_p_expr_logical_precedence() {
        // a || b && c => a || (b && c)
        let mut input = Input::new("true || false && true");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Or,
                left: Box::new(ast::Expr::LBool(true)),
                right: Box::new(ast::Expr::Binary {
                    operator: ast::OpBin::And,
                    left: Box::new(ast::Expr::LBool(false)),
                    right: Box::new(ast::Expr::LBool(true)),
                }),
            }
        );

        // !a && b => (!a) && b
        let mut input = Input::new("!true && false");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::And,
                left: Box::new(ast::Expr::Unary {
                    operator: ast::OpUn::Not,
                    operand: Box::new(ast::Expr::LBool(true)),
                }),
                right: Box::new(ast::Expr::LBool(false)),
            }
        );

        // a == b && c => (a == b) && c   (comparison binds tighter than &&)
        let mut input = Input::new("1 == 2 && true");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::And,
                left: Box::new(ast::Expr::Binary {
                    operator: ast::OpBin::Eq,
                    left: Box::new(ast::Expr::LNumber(1.0)),
                    right: Box::new(ast::Expr::LNumber(2.0)),
                }),
                right: Box::new(ast::Expr::LBool(true)),
            }
        );

        // != still parses as infix Neq, not !(=...)
        let mut input = Input::new("1 != 2");
        let res = p_expr.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Expr::Binary {
                operator: ast::OpBin::Neq,
                left: Box::new(ast::Expr::LNumber(1.0)),
                right: Box::new(ast::Expr::LNumber(2.0)),
            }
        );
    }

    #[test]
    fn test_p_match() {
        // Full match with three arms: guard, literal, binding.
        let mut input = Input::new("match x { x if x > 10 => 100, 2 => 99, y => y + 1 }");
        let res = p_expr.parse_next(&mut input).unwrap();
        let (scrutinee, arms) = match res {
            ast::Expr::Match { scrutinee, arms } => (scrutinee, arms),
            other => panic!("Expected Match, got {:?}", other),
        };
        assert_eq!(*scrutinee, ast::Expr::Variable("x".to_string()));
        assert_eq!(arms.len(), 3);

        // arm 0: binding `x` with guard `x > 10` body `100`
        assert_eq!(arms[0].pattern, ast::Pattern::Binding("x".to_string()));
        assert_eq!(
            arms[0].guard,
            Some(ast::Expr::Binary {
                operator: ast::OpBin::Gt,
                left: Box::new(ast::Expr::Variable("x".to_string())),
                right: Box::new(ast::Expr::LNumber(10.0)),
            })
        );
        assert_eq!(arms[0].body, ast::Expr::LNumber(100.0));

        // arm 1: literal 2, no guard
        assert_eq!(
            arms[1].pattern,
            ast::Pattern::Literal(ast::Expr::LNumber(2.0))
        );
        assert_eq!(arms[1].guard, None);
        assert_eq!(arms[1].body, ast::Expr::LNumber(99.0));

        // arm 2: binding `y`, body `y + 1`
        assert_eq!(arms[2].pattern, ast::Pattern::Binding("y".to_string()));
        assert_eq!(arms[2].guard, None);
        assert_eq!(
            arms[2].body,
            ast::Expr::Binary {
                operator: ast::OpBin::Add,
                left: Box::new(ast::Expr::Variable("y".to_string())),
                right: Box::new(ast::Expr::LNumber(1.0)),
            }
        );
    }

    #[test]
    fn test_p_if() {
        let mut input = Input::new("if x > 10 { 100 } else { x + 1 }");
        let res = p_expr.parse_next(&mut input).unwrap();
        let (condition, then_branch, else_branch) = match res {
            ast::Expr::If {
                condition,
                then_branch,
                else_branch,
            } => (condition, then_branch, else_branch),
            other => panic!("Expected If, got {:?}", other),
        };
        assert_eq!(
            *condition,
            ast::Expr::Binary {
                operator: ast::OpBin::Gt,
                left: Box::new(ast::Expr::Variable("x".to_string())),
                right: Box::new(ast::Expr::LNumber(10.0)),
            }
        );
        assert_eq!(*then_branch, ast::Expr::LNumber(100.0));
        assert_eq!(
            *else_branch,
            ast::Expr::Binary {
                operator: ast::OpBin::Add,
                left: Box::new(ast::Expr::Variable("x".to_string())),
                right: Box::new(ast::Expr::LNumber(1.0)),
            }
        );
    }

    #[test]
    fn test_p_if_nested() {
        let mut input = Input::new("if a { if b { 1 } else { 2 } } else { 3 }");
        let res = p_expr.parse_next(&mut input).unwrap();
        match res {
            ast::Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                assert_eq!(*condition, ast::Expr::Variable("a".to_string()));
                assert!(matches!(*then_branch, ast::Expr::If { .. }));
                assert_eq!(*else_branch, ast::Expr::LNumber(3.0));
            }
            other => panic!("Expected If, got {:?}", other),
        }
    }

    #[test]
    fn test_p_match_single_arm() {
        let mut input = Input::new("match x { 2 => 99 }");
        let res = p_expr.parse_next(&mut input).unwrap();
        match res {
            ast::Expr::Match { arms, .. } => {
                assert_eq!(arms.len(), 1);
                assert_eq!(
                    arms[0].pattern,
                    ast::Pattern::Literal(ast::Expr::LNumber(2.0))
                );
                assert!(arms[0].guard.is_none());
            }
            other => panic!("Expected Match, got {:?}", other),
        }
    }
}
