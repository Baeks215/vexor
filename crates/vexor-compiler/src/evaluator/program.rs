//! Evaluator for program

use itertools::Itertools;

use crate::evaluator::expr::Callable;
use crate::evaluator::{EResult, EnvExt, EnvRef, Value, expr, ty};
use crate::ir::{ast, scene};

/// Evaluates a program, returns the result of the last expression.
pub fn eval_program(program: ast::Program) -> EResult<scene::Scene> {
    let env = EnvRef::empty();
    let mut settings = scene::Settings {
        canvas: (1000, 1000), // Default canvas size
    };
    let ast::Program { units } = program;

    let mut exports: Vec<ast::Expr> = Vec::new();
    for unit in units {
        match unit {
            ast::ProgramUnit::Function { identifier, func } => {
                if !func.params.iter().all_unique() {
                    return Err(format!(
                        "function {identifier} has duplicate parameter names"
                    ));
                }
                let func = Callable::User {
                    func,
                    closure_env: env.clone(), // Clone reference,
                };
                env.set_var(identifier, Value::from(func))?;
            }
            ast::ProgramUnit::Assignment { identifier, value } => {
                env.set_var_lazy(identifier, value)?;
            }
            ast::ProgramUnit::Export(e) => {
                exports.push(e);
            }
            ast::ProgramUnit::Setting(setting) => match setting {
                ast::Setting::Canvas { width, height } => settings.canvas = (width, height),
            },
        }
    }

    let exported: Vec<_> = exports
        .into_iter()
        .map(|e| expr::eval::<ty::Graphic>(&env, e))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(scene::Scene {
        exports: exported,
        settings,
    })
}
