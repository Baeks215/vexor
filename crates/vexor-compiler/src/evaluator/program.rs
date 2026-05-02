//! Evaluator for program

use crate::evaluator::{Context, EResult, expr};
use crate::ir::scene;
use crate::ir::typed;

pub fn eval_assignment(context: &mut Context, statement: typed::Assignment) -> EResult<()> {
    match statement {
        typed::Assignment { identifier, value } => {
            let evaluated = expr::eval_generic(context, value)?;
            let old = context.set_var(identifier, evaluated);
            if let Some(_) = old {
                return Err("Variable already exists".to_string());
            }
            Ok(())
        }
    }
}

/// Evaluates a program, returns the result of the last expression.
pub fn eval_program(program: typed::Program) -> EResult<scene::Scene> {
    let mut context = Context::new();
    let mut exported: Vec<scene::Graphic> = Vec::new();

    let typed::Program {
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
        let evaluated = expr::eval_graphic(&context, export)?;
        exported.push(evaluated);
    }
    Ok(scene::Scene { exports: exported })
}
