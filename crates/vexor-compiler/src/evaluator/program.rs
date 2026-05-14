//! Evaluator for program

use itertools::Itertools;

use crate::evaluator::expr::Callable;
use crate::evaluator::{EResult, EnvExt, EnvRef, Value, expr, ty};
use crate::ir::{ast, scene};

pub fn eval_assignment(env: &EnvRef, identifier: String, value: ast::Expr) -> EResult<()> {
    let evaluated = expr::eval::<ty::Any>(env, value)?;
    env.set_var(identifier, evaluated)
}

/// Evaluates a program, returns the result of the last expression.
pub fn eval_program(program: ast::Program) -> EResult<scene::Scene> {
    let env = EnvRef::empty();
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
                env.set_var(identifier, Value::Function(Callable::User(func)))?;
            }
            ast::ProgramUnit::Assignment { identifier, value } => {
                eval_assignment(&env, identifier, value)?;
            }
            ast::ProgramUnit::Export(export) => {
                let evaluated = expr::eval::<ty::Graphic>(&env, export)?;
                exported.push(evaluated);
            }
        }
    }

    Ok(scene::Scene { exports: exported })
}
