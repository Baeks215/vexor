//! Type checker for program

use crate::ir::ast;
use crate::ir::typed;
use crate::type_checker::expr;
use crate::type_checker::{Context, FunctionType, TResult};

fn check_statement(
    context: &mut Context,
    assignment: ast::Assignment,
) -> TResult<typed::Assignment> {
    match assignment {
        ast::Assignment {
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
            Ok(typed::Assignment {
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
        scope,
        return_expr: (return_expr, return_ty),
    } = function;
    let mut inner = context.new_scope_function(&params);

    let scope = scope
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
        scope,
        return_expr,
    })
}

pub fn check_program(program: ast::Program) -> TResult<typed::Program> {
    let mut context = Context::new();
    let ast::Program {
        functions,
        scope: statements,
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
        .map(|expr| expr::check(&context, expr))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(typed::Program {
        functions,
        scope: new_statements,
        exports,
    })
}
