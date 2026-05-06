//! Evaluator for program

use crate::evaluator::{Context, EResult, expr};
use crate::ir::scene::marker;
use crate::ir::{ast, scene};

pub fn eval_assignment(context: &mut Context, statement: ast::Assignment) -> EResult<()> {
    match statement {
        ast::Assignment { identifier, value } => {
            let evaluated = expr::eval::<marker::Any>(context, value)?;
            let old = context.set_var(identifier, evaluated);
            if let Some(_) = old {
                return Err("Variable already exists".to_string());
            }
            Ok(())
        }
    }
}

/// Evaluates a program, returns the result of the last expression.
pub fn eval_program(program: ast::Program) -> EResult<scene::Scene> {
    let mut context = Context::new();
    let mut exported: Vec<scene::Graphic> = Vec::new();

    let ast::Program {
        functions,
        scope,
        exports,
    } = program;

    for func in functions {
        context.add_function(func);
    }

    for assignment in scope {
        eval_assignment(&mut context, assignment)?;
    }
    for export in exports {
        let evaluated = expr::eval::<marker::Graphic>(&context, export)?;
        exported.push(evaluated);
    }
    Ok(scene::Scene { exports: exported })
}
