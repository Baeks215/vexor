//! Parser: Text -> AST

use crate::ir::ast;
use crate::parser::common::keyword::{pk_export, pk_let};
use crate::parser::common::{Input, lexeme, p_identifier};
use crate::parser::expr::p_expr;
use winnow::combinator::{alt, preceded};
use winnow::{ModalResult, Parser};

mod common;
mod expr;
mod graphic;

fn p_statement<'a>(input: &mut Input<'a>) -> ModalResult<ast::Statement> {
    alt((
        (pk_let, p_identifier, lexeme("="), p_expr).map(|(_, i, _, e)| {
            ast::Statement::Assignment {
                identifier: i.to_string(),
                value: e,
            }
        }),
        preceded(pk_export, p_expr).map(|e| ast::Statement::Export { graphic: e }),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p_statement_assignment() {
        let mut input = Input::new("let x = 10");
        let res = p_statement.parse_next(&mut input).unwrap();
        if let ast::Statement::Assignment { identifier, value } = res {
            assert_eq!(identifier, "x");
            assert_eq!(value, ast::Expr::LNumber(10.0));
        } else {
            panic!("Expected Assignment, got {:?}", res);
        }

        let mut input = Input::new("let my_var = \"hello\"");
        let res = p_statement.parse_next(&mut input).unwrap();
        if let ast::Statement::Assignment { identifier, value } = res {
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
}
