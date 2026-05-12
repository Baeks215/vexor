//! Evaluator for program

use crate::evaluator::{Context, EResult, expr, ty};
use crate::ir::{ast, scene};

pub fn eval_assignment(context: &mut Context, statement: ast::Assignment) -> EResult<()> {
    match statement {
        ast::Assignment { identifier, value } => {
            let evaluated = expr::eval::<ty::Any>(context, value)?;
            let old = context.set_var(identifier, evaluated);
            if let Some(_) = old {
                return Err("variable already exists".to_string());
            }
            Ok(())
        }
    }
}

/// Evaluates a program, returns the result of the last expression.
pub fn eval_program(program: ast::Program) -> EResult<scene::Scene> {
    let mut context = Context::new();
    let ast::Program { units } = program;

    let mut exported: Vec<scene::Graphic> = Vec::new();
    for unit in units {
        match unit {
            ast::ProgramUnit::Function(func) => {
                context.add_function(func);
            }
            ast::ProgramUnit::Assignment(assignment) => {
                eval_assignment(&mut context, assignment)?;
            }
            ast::ProgramUnit::Export(export) => {
                let evaluated = expr::eval::<ty::Graphic>(&context, export)?;
                exported.push(evaluated);
            }
        }
    }

    Ok(scene::Scene { exports: exported })
}
