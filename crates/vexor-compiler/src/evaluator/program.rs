//! Evaluator for program

use itertools::Itertools;

use crate::evaluator::{Context, EResult, Value, expr, ty};
use crate::ir::{ast, scene};

pub fn eval_assignment(context: &mut Context, identifier: String, value: ast::Expr) -> EResult<()> {
    let evaluated = expr::eval::<ty::Any>(context, value)?;
    context.set_var(identifier, evaluated)
}

/// Evaluates a program, returns the result of the last expression.
pub fn eval_program(program: ast::Program) -> EResult<scene::Scene> {
    let mut context = Context::new();
    let ast::Program { units } = program;

    let mut exported: Vec<scene::Graphic> = Vec::new();
    for unit in units {
        match unit {
            ast::ProgramUnit::Function { identifier, func } => {
                if !func.params.iter().all_unique() {
                    return Err(format!(
                        "function {identifier} has duplicate parameter names"
                    ));
                }
                context.set_var(identifier, Value::Function(func))?;
            }
            ast::ProgramUnit::Assignment { identifier, value } => {
                eval_assignment(&mut context, identifier, value)?;
            }
            ast::ProgramUnit::Export(export) => {
                let evaluated = expr::eval::<ty::Graphic>(&context, export)?;
                exported.push(evaluated);
            }
        }
    }

    Ok(scene::Scene { exports: exported })
}
