//! Parser: Text -> AST

use crate::ir::ast;
use crate::ir::typed::Type;
use crate::parser::expr::p_expr;
use crate::parser::keyword::{pk_color, pk_export, pk_graphic, pk_let, pk_number, pk_string};
use crate::parser::{Input, lexeme, p_identifier};
use winnow::ascii::{line_ending, multispace0};
use winnow::combinator::{alt, delimited, preceded, separated};
use winnow::error::{ContextError, ParseError};
use winnow::{ModalResult, Parser, Result};

fn p_statement<'a>(input: &mut Input<'a>) -> ModalResult<ast::Statement> {
    alt((
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
            }),
        preceded(pk_export, p_expr).map(|e| ast::Statement::Export { graphic: e }),
    ))
    .parse_next(input)
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

/// Parses a program from the given input string.
///   Text -> AST
pub fn parse_program<'a>(
    input: &'a str,
) -> Result<ast::Program, ParseError<Input<'a>, ContextError>> {
    let input = Input::new(input);
    delimited(
        multispace0,
        separated(0.., p_statement, lexeme(line_ending))
            .map(|stmts| ast::Program { statements: stmts }),
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
        if let ast::Statement::Assignment {
            ty,
            identifier,
            value,
        } = res
        {
            assert_eq!(ty, Type::Number);
            assert_eq!(identifier, "x");
            assert_eq!(value, ast::Expr::LNumber(10.0));
        } else {
            panic!("Expected Assignment, got {:?}", res);
        }

        let mut input = Input::new("let my_var: string = \"hello\"");
        let res = p_statement.parse_next(&mut input).unwrap();
        if let ast::Statement::Assignment {
            ty,
            identifier,
            value,
        } = res
        {
            assert_eq!(ty, Type::String);
            assert_eq!(identifier, "my_var");
            assert_eq!(value, ast::Expr::LString("hello".to_string()));
        } else {
            panic!("Expected Assignment, got {:?}", res);
        }
    }

    #[test]
    fn test_p_statement_export() {
        let mut input = Input::new("export circle(10)");
        let res = p_statement.parse_next(&mut input).unwrap();
        if let ast::Statement::Export { graphic } = res {
            match graphic {
                ast::Expr::LGraphic(ast::Graphic::Circle { radius }) => {
                    assert_eq!(*radius, ast::Expr::LNumber(10.0));
                }
                _ => panic!("Expected Circle graphic, got {:?}", graphic),
            }
        } else {
            panic!("Expected Export, got {:?}", res);
        }
    }

    #[test]
    fn test_parse_program() {
        let input = "  let x: number = 10  \n \t export circle(x)  \n";
        let res = parse_program(input).unwrap();
        assert_eq!(res.statements.len(), 2);

        if let ast::Statement::Assignment {
            ty,
            identifier,
            value,
        } = &res.statements[0]
        {
            assert_eq!(ty, &Type::Number);
            assert_eq!(identifier, "x");
            assert_eq!(*value, ast::Expr::LNumber(10.0));
        } else {
            panic!("Expected Assignment, got {:?}", res.statements[0]);
        }

        if let ast::Statement::Export { graphic } = &res.statements[1] {
            match graphic {
                ast::Expr::LGraphic(ast::Graphic::Circle { radius }) => {
                    assert_eq!(**radius, ast::Expr::Variable("x".to_string()));
                }
                _ => panic!("Expected Circle graphic, got {:?}", graphic),
            }
        } else {
            panic!("Expected Export, got {:?}", res.statements[1]);
        }
    }
}
