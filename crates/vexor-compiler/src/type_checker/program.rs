use crate::ir::ast;
use crate::ir::typed::{Type, ast as typed_ast};
use crate::type_checker::expr::check_expr;
use crate::type_checker::{Constraint, Context, TResult};
use Constraint::*;

fn check_statement(
    context: &mut Context,
    statement: ast::Statement,
) -> TResult<typed_ast::Statement> {
    match statement {
        ast::Statement::Assignment { identifier, value } => {
            let typed_expr = check_expr(&context, value, Any)?;
            // Set variable in context.
            let old = context.var_types.insert(identifier.clone(), typed_expr.ty);
            // Ensure variable does not already exist.
            if let Some(_) = old {
                return Err(format!("Variable '{}' already exists", identifier));
            }
            Ok(typed_ast::Statement::Assignment {
                identifier,
                value: typed_expr,
            })
        }
        ast::Statement::Export { graphic } => check_expr(&context, graphic, Is(Type::Graphic))
            .map(|expr| typed_ast::Statement::Export { graphic: expr }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_statement_assignment() {
        let mut context = Context::new();
        let statement = ast::Statement::Assignment {
            identifier: "x".to_string(),
            value: ast::Expr::LNumber(10.0),
        };

        // Success
        let res = check_statement(&mut context, statement).unwrap();
        if let typed_ast::Statement::Assignment { identifier, value } = res {
            assert_eq!(identifier, "x");
            assert_eq!(value.ty, Type::Number);
        } else {
            panic!("Expected Assignment, got {:?}", res);
        }

        // Failure: already exists
        let mut context = Context::new();
        context.var_types.insert("x".to_string(), Type::Number);
        let statement = ast::Statement::Assignment {
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
        if let typed_ast::Statement::Export { graphic } = res {
            assert_eq!(graphic.ty, Type::Graphic);
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
}
