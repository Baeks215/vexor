//! Evaluator for program

use crate::evaluator::{Context, EResult, expr};
use crate::ir::scene;
use crate::ir::typed;

/// Evaluates a statement, returns graphic if it's an export statement.
fn eval_statement(
    context: &mut Context,
    statement: typed::Statement,
) -> EResult<Option<scene::Graphic>> {
    match statement {
        typed::Statement::Assignment { identifier, value } => {
            let evaluated = expr::eval_generic(context, value)?;
            let old = context.set_var(identifier, evaluated);
            if let Some(_) = old {
                return Err("Variable already exists".to_string());
            }
            Ok(None)
        }
        typed::Statement::Export { graphic } => {
            let evaluated = expr::eval_graphic(context, graphic)?;
            Ok(Some(evaluated))
        }
    }
}

/// Evaluates a program, returns the result of the last expression.
pub fn eval_program(program: typed::Program) -> EResult<scene::Scene> {
    let mut context = Context::new();
    let mut exported: Vec<scene::Graphic> = Vec::new();

    for statement in &program.statements {
        let evaluated = eval_statement(&mut context, statement.clone())?;
        if let Some(graphic) = evaluated {
            exported.push(graphic);
        }
    }
    Ok(scene::Scene { exports: exported })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::typed::expr::*;

    #[test]
    fn test_eval_statement_export() {
        let mut context = Context::new();
        let stmt = typed::Statement::Export {
            graphic: Expr::Node(crate::ir::typed::Graphic::Circle {
                radius: Box::new(Expr::Node(NodeNumber::Literal(5.0))),
            }),
        };

        let res = eval_statement(&mut context, stmt).unwrap().unwrap();
        assert_eq!(res, scene::Graphic::Circle { radius: 5.0 });
    }

    #[test]
    fn test_eval_statement_already_exists() {
        let mut context = Context::new();
        let stmt1 = typed::Statement::Assignment {
            identifier: "x".to_string(),
            value: ExprGeneric::Number(Expr::Node(NodeNumber::Literal(1.0))),
        };
        eval_statement(&mut context, stmt1).unwrap();

        let stmt2 = typed::Statement::Assignment {
            identifier: "x".to_string(),
            value: ExprGeneric::Number(Expr::Node(NodeNumber::Literal(2.0))),
        };
        assert!(eval_statement(&mut context, stmt2).is_err());
    }

    #[test]
    fn test_eval_program() {
        let program = typed::Program {
            statements: vec![
                typed::Statement::Assignment {
                    identifier: "r".to_string(),
                    value: ExprGeneric::Number(Expr::Node(NodeNumber::Literal(10.0))),
                },
                typed::Statement::Export {
                    graphic: Expr::Node(crate::ir::typed::Graphic::Circle {
                        radius: Box::new(Expr::Variable("r".to_string())),
                    }),
                },
            ],
        };

        let scene = eval_program(program).unwrap();
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], scene::Graphic::Circle { radius: 10.0 });
    }
}
