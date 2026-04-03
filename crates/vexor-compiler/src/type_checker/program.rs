//! Type checker for program

use crate::ir::ast;
use crate::ir::typed::{self, Type};
use crate::type_checker::expr;
use crate::type_checker::{Constraint, Context, TResult};
use Constraint::*;

fn check_statement(context: &mut Context, statement: ast::Statement) -> TResult<typed::Statement> {
    match statement {
        ast::Statement::Assignment {
            ty,
            identifier,
            value,
        } => {
            let typed_expr = expr::check_generic(&context, ty, value)?;
            // Set variable in context.
            let old = context.set_var(identifier.clone(), ty);
            // Ensure variable does not already exist.
            if let Some(_) = old {
                return Err(format!("Variable '{}' already exists", identifier));
            }
            Ok(typed::Statement::Assignment {
                identifier,
                value: typed_expr,
            })
        }
        ast::Statement::Export { graphic } => expr::check_graphic(&context, graphic)
            .map(|expr| typed::Statement::Export { graphic: expr }),
    }
}

pub fn check_program(program: ast::Program) -> TResult<typed::Program> {
    let mut context = Context::new();
    let ast::Program { statements } = program;
    let mut new_statements = Vec::new();
    for statement in statements {
        let typed_statement = check_statement(&mut context, statement)?;
        new_statements.push(typed_statement);
    }
    Ok(typed::Program {
        statements: new_statements,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_statement_assignment() {
        let mut context = Context::new();
        let statement = ast::Statement::Assignment {
            ty: Type::Number,
            identifier: "x".to_string(),
            value: ast::Expr::LNumber(10.0),
        };

        // Success
        let res = check_statement(&mut context, statement).unwrap();
        if let typed::Statement::Assignment { identifier, value } = res {
            assert_eq!(identifier, "x");
            assert!(matches!(value, typed::expr::ExprGeneric::Number(_)));
        } else {
            panic!("Expected Assignment, got {:?}", res);
        }

        // Failure: already exists
        let mut context = Context::new();
        context.set_var("x".to_string(), Type::Number);
        let statement = ast::Statement::Assignment {
            ty: Type::Number,
            identifier: "x".to_string(),
            value: ast::Expr::LNumber(20.0),
        };
        assert!(check_statement(&mut context, statement).is_err());
    }

    #[test]
    fn test_check_statement_export() {
        let mut context = Context::new();

        // Success: export a graphic
        let statement = ast::Statement::Export {
            graphic: ast::Expr::LGraphic(ast::Graphic::Circle {
                radius: Box::new(ast::Expr::LNumber(10.0)),
            }),
        };
        let res = check_statement(&mut context, statement).unwrap();
        if let typed::Statement::Export { graphic } = res {
            assert!(matches!(graphic, typed::expr::ExprGraphic::Node(_)));
        } else {
            panic!("Expected Export, got {:?}", res);
        }

        // Failure: export a number
        let mut context = Context::new();
        let statement = ast::Statement::Export {
            graphic: ast::Expr::LNumber(10.0),
        };
        assert!(check_statement(&mut context, statement).is_err());
    }

    #[test]
    fn test_check_program() {
        // Success: export a graphic with a number variable
        let program = ast::Program {
            statements: vec![
                ast::Statement::Assignment {
                    ty: Type::Number,
                    identifier: "x".to_string(),
                    value: ast::Expr::LNumber(10.0),
                },
                ast::Statement::Export {
                    graphic: ast::Expr::LGraphic(ast::Graphic::Circle {
                        radius: Box::new(ast::Expr::Variable("x".to_string())),
                    }),
                },
            ],
        };
        let res = check_program(program).unwrap();
        assert_eq!(res.statements.len(), 2);

        // Test failure (e.g. re-assignment)
        let program = ast::Program {
            statements: vec![
                ast::Statement::Assignment {
                    ty: Type::Number,
                    identifier: "x".to_string(),
                    value: ast::Expr::LNumber(10.0),
                },
                ast::Statement::Assignment {
                    ty: Type::Number,
                    identifier: "x".to_string(),
                    value: ast::Expr::LNumber(20.0),
                },
            ],
        };
        assert!(check_program(program).is_err());
    }
}
