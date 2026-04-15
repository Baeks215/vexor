//! Parser: Text -> AST

use crate::ir::ast;
use crate::ir::typed::Type;
use crate::parser::expr::p_expr;
use crate::parser::keyword::{
    pk_color, pk_export, pk_fn, pk_graphic, pk_let, pk_number, pk_return, pk_string,
};
use crate::parser::{Input, bracketed, lexeme, ml_lexeme, p_identifier};
use itertools::{Either, Itertools};
use winnow::ascii::{line_ending, multispace0};
use winnow::combinator::{alt, delimited, preceded, separated, separated_pair};
use winnow::error::{ContextError, ParseError};
use winnow::{ModalResult, Parser, Result};

#[derive(Debug, Clone, PartialEq)]
enum ProgramUnit {
    Statement(ast::Statement),
    Function(ast::Function),
}

fn p_type<'a>(input: &mut Input<'a>) -> ModalResult<Type> {
    alt((
        pk_number.map(|_| Type::Number),
        pk_string.map(|_| Type::String),
        pk_color.map(|_| Type::Color),
        pk_graphic.map(|_| Type::Graphic),
    ))
    .parse_next(input)
}

fn p_statement<'a>(input: &mut Input<'a>) -> ModalResult<ast::Statement> {
    (
        pk_let,
        p_identifier,
        lexeme(":"),
        p_type,
        lexeme("="),
        p_expr,
    )
        .map(|(_, i, _, t, _, e)| ast::Statement::Assignment {
            ty: t,
            identifier: i.to_string(),
            value: e,
        })
        .parse_next(input)
}

fn p_function<'a>(input: &mut Input<'a>) -> ModalResult<ast::Function> {
    (
        pk_fn,
        p_identifier,
        bracketed(separated(
            0..,
            separated_pair(p_identifier, lexeme(":"), p_type),
            lexeme(","),
        )),
        lexeme(":"),
        p_type,
        delimited(
            ml_lexeme("{"),
            separated_pair(
                separated(0.., p_statement, lexeme(line_ending)),
                multispace0,
                delimited(pk_return, p_expr, multispace0),
            ),
            lexeme("}"),
        ),
    )
        .map(
            |(_, name, params, _, return_type, (body, return_expr)): (
                _,
                _,
                Vec<(&str, Type)>,
                _,
                _,
                _,
            )| ast::Function {
                name: name.to_string(),
                params: params
                    .into_iter()
                    .map(|(n, t)| (n.to_string(), t))
                    .collect(),
                body,
                return_expr: (return_expr, return_type),
            },
        )
        .parse_next(input)
}

fn p_program_unit<'a>(input: &mut Input<'a>) -> ModalResult<ProgramUnit> {
    alt((
        p_function.map(|f| ProgramUnit::Function(f)),
        p_statement.map(|s| ProgramUnit::Statement(s)),
    ))
    .parse_next(input)
}

fn p_export<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    preceded(pk_export, p_expr).parse_next(input)
}

/// Parses a program from the given input string.
///   Text -> AST
pub fn parse_program<'a>(
    input: &'a str,
) -> Result<ast::Program, ParseError<Input<'a>, ContextError>> {
    let input = Input::new(input);
    delimited(
        multispace0,
        separated_pair(
            separated(0.., p_program_unit, lexeme(line_ending)),
            multispace0,
            separated(0.., p_export, lexeme(line_ending)),
        )
        .map(|(units, exports): (Vec<ProgramUnit>, Vec<ast::Expr>)| {
            let (functions, statements) = units.into_iter().partition_map(|u| match u {
                ProgramUnit::Function(f) => Either::Left(f),
                ProgramUnit::Statement(s) => Either::Right(s),
            });
            ast::Program {
                functions,
                statements,
                exports,
            }
        }),
        multispace0,
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p_statement_assignment() {
        let mut input = Input::new("let x: number = 10");
        let res = p_statement.parse_next(&mut input).unwrap();
        let ast::Statement::Assignment {
            ty,
            identifier,
            value,
        } = res;
        assert_eq!(ty, Type::Number);
        assert_eq!(identifier, "x");
        assert_eq!(value, ast::Expr::LNumber(10.0));

        let mut input = Input::new("let my_var: string = \"hello\"");
        let res = p_statement.parse_next(&mut input).unwrap();
        let ast::Statement::Assignment {
            ty,
            identifier,
            value,
        } = res;
        assert_eq!(ty, Type::String);
        assert_eq!(identifier, "my_var");
        assert_eq!(value, ast::Expr::LString("hello".to_string()));
    }

    #[test]
    fn test_p_export() {
        let mut input = Input::new("export circle(10)");
        let res = p_export.parse_next(&mut input).unwrap();
        if let ast::Expr::LGraphic(ast::Graphic::Circle { radius }) = res {
            assert_eq!(*radius, ast::Expr::LNumber(10.0));
        } else {
            panic!("Expected Export, got {:?}", res);
        }
    }

    #[test]
    fn test_p_function() {
        let input = "fn double(x: number): number {\nlet y: number = x + x\nreturn y\n}";
        let mut input = Input::new(input);
        let res = p_function.parse_next(&mut input).unwrap();
        assert_eq!(res.name, "double");
        assert_eq!(res.params, vec![("x".to_string(), Type::Number)]);
        assert_eq!(res.return_expr.1, Type::Number);
        assert_eq!(res.body.len(), 1);
        let ast::Statement::Assignment { identifier, .. } = &res.body[0];
        assert_eq!(identifier, "y");

        // Zero-param, empty-body function
        let mut input = Input::new("fn five(): number { return 5 }");
        let res = p_function.parse_next(&mut input).unwrap();
        assert_eq!(res.name, "five");
        assert!(res.params.is_empty());
        assert!(res.body.is_empty());
        assert_eq!(res.return_expr.0, ast::Expr::LNumber(5.0));
    }

    #[test]
    fn test_parse_program_with_function() {
        let input = "fn mk(r: number): number { return r + 1 }\nexport circle(mk(5))";
        let res = parse_program(input).unwrap();
        assert_eq!(res.functions.len(), 1);
        assert!(res.statements.is_empty());
        assert_eq!(res.exports.len(), 1);

        if let ast::Expr::LGraphic(ast::Graphic::Circle { radius }) = &res.exports[0] {
            match &**radius {
                ast::Expr::Call { function, args } => {
                    assert_eq!(function, "mk");
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], ast::Expr::LNumber(5.0));
                }
                other => panic!("Expected Call, got {:?}", other),
            }
        } else {
            panic!("Expected Circle export, got {:?}", res.exports[0]);
        }
    }

    #[test]
    fn test_parse_program() {
        let input = "  let x: number = 10  \n \t export circle(x)  \n";
        let res = parse_program(input).unwrap();
        assert_eq!(res.statements.len(), 1);

        let ast::Statement::Assignment {
            ty,
            identifier,
            value,
        } = &res.statements[0];
        assert_eq!(ty, &Type::Number);
        assert_eq!(identifier, "x");
        assert_eq!(*value, ast::Expr::LNumber(10.0));

        if let ast::Expr::LGraphic(ast::Graphic::Circle { radius }) = &res.exports[0] {
            assert_eq!(**radius, ast::Expr::Variable("x".to_string()));
        } else {
            panic!("Expected Export, got {:?}", res.exports[0]);
        }
    }
}
