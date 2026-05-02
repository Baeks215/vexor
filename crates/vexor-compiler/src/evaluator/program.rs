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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::typed::{ColorT, NumberT, expr::*};

    #[test]
    fn test_eval_statement() {
        let mut context = Context::new();
        let stmt1 = typed::Assignment {
            identifier: "x".to_string(),
            value: ExprGeneric::Number(Expr::Literal(1.0)),
        };
        eval_assignment(&mut context, stmt1).unwrap();

        let stmt2 = typed::Assignment {
            identifier: "x".to_string(),
            value: ExprGeneric::Number(Expr::Literal(2.0)),
        };
        assert!(eval_assignment(&mut context, stmt2).is_err());
    }

    fn red_color_expr() -> Box<Expr<ColorT>> {
        Box::new(Expr::Literal(typed::Color::Rgba {
            r: Box::new(Expr::Literal(1.0)),
            g: Box::new(Expr::Literal(0.0)),
            b: Box::new(Expr::Literal(0.0)),
            a: Box::new(Expr::Literal(1.0)),
        }))
    }

    fn red_scene() -> scene::Color {
        scene::Color::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    fn zero() -> Box<Expr<NumberT>> {
        Box::new(Expr::Literal(0.0))
    }

    #[test]
    fn test_eval_program() {
        let program = typed::Program {
            functions: vec![],
            scope: vec![typed::Assignment {
                identifier: "r".to_string(),
                value: ExprGeneric::Number(Expr::Literal(10.0)),
            }],
            exports: vec![Expr::Literal(crate::ir::typed::Graphic::Circle {
                x: zero(),
                y: zero(),
                radius: Box::new(Expr::Variable("r".to_string())),
                color: red_color_expr(),
            })],
        };

        let scene = eval_program(program).unwrap();
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(
            scene.exports[0],
            scene::Graphic::Circle {
                x: 0.0,
                y: 0.0,
                radius: 10.0,
                color: red_scene()
            }
        );
    }

    #[test]
    fn test_eval_program_with_function() {
        let double = typed::Function {
            name: "double".to_string(),
            params: vec![("x".to_string(), typed::Type::Number)],
            scope: vec![],
            return_expr: ExprGeneric::Number(Expr::Operator(NumberOps::Arithmetic {
                op: ArithmeticOp::Add,
                left: Box::new(Expr::Variable("x".to_string())),
                right: Box::new(Expr::Variable("x".to_string())),
            })),
        };
        let program = typed::Program {
            functions: vec![double],
            scope: vec![],
            exports: vec![Expr::Literal(typed::Graphic::Circle {
                x: zero(),
                y: zero(),
                radius: Box::new(Expr::Call {
                    function: "double".to_string(),
                    arguments: vec![ExprGeneric::Number(Expr::Literal(7.0))],
                }),
                color: red_color_expr(),
            })],
        };
        let scene = eval_program(program).unwrap();
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(
            scene.exports[0],
            scene::Graphic::Circle {
                x: 0.0,
                y: 0.0,
                radius: 14.0,
                color: red_scene()
            }
        );
    }
}
