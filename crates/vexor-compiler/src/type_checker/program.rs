//! Type checker for program

use crate::ir::ast;
use crate::ir::typed;
use crate::type_checker::expr;
use crate::type_checker::{Context, FunctionType, TResult};

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
    }
}

fn check_function(context: &mut Context, function: ast::Function) -> TResult<typed::Function> {
    let ast::Function {
        name,
        params,
        body,
        return_expr: (return_expr, return_ty),
    } = function;
    let mut inner = context.new_scope_function(&params);

    let body = body
        .into_iter()
        .map(|s| check_statement(&mut inner, s))
        .collect::<Result<Vec<_>, _>>()?;
    let return_expr = expr::check_generic(&inner, return_ty, return_expr)?;
    context.add_function(
        name.clone(),
        FunctionType {
            args: params.clone().into_iter().map(|(_, ty)| ty).collect(),
            return_type: return_ty,
        },
    );

    Ok(typed::Function {
        name,
        params,
        body,
        return_expr,
    })
}

pub fn check_program(program: ast::Program) -> TResult<typed::Program> {
    let mut context = Context::new();
    let ast::Program {
        functions,
        statements,
        exports,
    } = program;
    let functions = functions
        .into_iter()
        .map(|f| check_function(&mut context, f))
        .collect::<Result<Vec<_>, _>>()?;

    let mut new_statements = Vec::new();
    for statement in statements {
        let typed_statement = check_statement(&mut context, statement)?;
        new_statements.push(typed_statement);
    }

    let exports = exports
        .into_iter()
        .map(|expr| expr::check_graphic(&context, expr))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(typed::Program {
        functions,
        statements: new_statements,
        exports,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::typed::Type;

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
        let typed::Statement::Assignment { identifier, value } = res;
        assert_eq!(identifier, "x");
        assert!(matches!(value, typed::expr::ExprGeneric::Number(_)));

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
    fn test_check_program() {
        // Success: export a graphic with a number variable
        let program = ast::Program {
            functions: vec![],
            statements: vec![ast::Statement::Assignment {
                ty: Type::Number,
                identifier: "x".to_string(),
                value: ast::Expr::LNumber(10.0),
            }],
            exports: vec![ast::Expr::LGraphic(ast::Graphic::Circle {
                radius: Box::new(ast::Expr::Variable("x".to_string())),
            })],
        };
        let res = check_program(program).unwrap();
        assert_eq!(res.statements.len(), 1);
        assert_eq!(res.exports.len(), 1);

        // Test failure (e.g. re-assignment)
        let program = ast::Program {
            functions: vec![],
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
            exports: vec![],
        };
        assert!(check_program(program).is_err());
    }
}
