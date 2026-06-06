//! Evaluator for program

use itertools::Itertools;
use std::rc::Rc;

use crate::evaluator::expr::{Callable, Evaluable, eval};
use crate::evaluator::{EResult, EnvExt, EnvRef, Value, WithSpan, ty};
use crate::ir::ast::SpanExpr;
use crate::ir::{ast, scene};

enum ExportExpr {
    One(SpanExpr),
    Each(SpanExpr),
}

/// Evaluates a program, returns the result of the last expression.
pub fn eval_program(program: ast::Program) -> EResult<scene::Scene> {
    let env = EnvRef::top_level();
    let mut settings = scene::Settings::default();
    let ast::Program { units } = program;

    let mut export_exprs: Vec<ExportExpr> = vec![];
    for unit in units {
        let unit_span = unit.span.clone();
        match unit.node {
            ast::ProgramUnit::Function { identifier, func } => {
                if !func.params.iter().all_unique() {
                    return Err(ast::Spanned {
                        node: format!("function {identifier} has duplicate parameter names"),
                        span: unit_span,
                    });
                }
                let func = Callable::User {
                    func: Rc::new(func),
                    closure_env: env.clone(), // Clone reference,
                };
                env.set_var(identifier, Value::from(func))
                    .with_span_if_missing(unit_span)?;
            }
            ast::ProgramUnit::Assignment { identifier, value } => {
                env.set_var_lazy(identifier, value)
                    .with_span_if_missing(unit_span)?;
            }
            ast::ProgramUnit::Export(e) => {
                export_exprs.push(ExportExpr::One(e));
            }
            ast::ProgramUnit::ExportEach(e) => {
                export_exprs.push(ExportExpr::Each(e));
            }
            ast::ProgramUnit::Setting(setting) => match setting {
                ast::Setting::Canvas { width, height } => settings.canvas = (width, height),
                ast::Setting::Precision(p) => settings.precision = p,
            },
        }
    }

    let mut exports: Vec<scene::Graphic> = vec![];
    for e in export_exprs {
        match e {
            ExportExpr::One(e) => {
                let g = eval::<ty::Graphic>(&env, &e)?;
                exports.push(g);
            }
            ExportExpr::Each(es) => {
                let l = eval::<ty::List>(&env, &es)?;
                for v in l.into_iter() {
                    let g = ty::Graphic::expect(v)?;
                    exports.push(g);
                }
            }
        }
    }

    Ok(scene::Scene { exports, settings })
}
