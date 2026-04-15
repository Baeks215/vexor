//! Evaluator for program

use crate::evaluator::{Context, EResult, expr};
use crate::ir::scene;
use crate::ir::typed;

pub fn eval_statement(context: &mut Context, statement: typed::Statement) -> EResult<()> {
    match statement {
        typed::Statement::Assignment { identifier, value } => {
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
        statements,
        exports,
    } = program;

    for func in functions {
        context.add_function(func);
    }

    for statement in statements {
        eval_statement(&mut context, statement)?;
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
    use crate::ir::typed::expr::*;

    #[test]
    fn test_eval_statement() {
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
            functions: vec![],
            statements: vec![typed::Statement::Assignment {
                identifier: "r".to_string(),
                value: ExprGeneric::Number(Expr::Node(NodeNumber::Literal(10.0))),
            }],
            exports: vec![Expr::Node(crate::ir::typed::Graphic::Circle {
                radius: Box::new(Expr::Variable("r".to_string())),
            })],
        };

        let scene = eval_program(program).unwrap();
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], scene::Graphic::Circle { radius: 10.0 });
    }

    #[test]
    fn test_eval_program_with_function() {
        // fn double(x: number): number { return x + x }
        // export circle(double(7))
        let double = typed::Function {
            name: "double".to_string(),
            params: vec![("x".to_string(), typed::Type::Number)],
            body: vec![],
            return_expr: ExprGeneric::Number(Expr::Node(NodeNumber::Binary {
                operator: OpBinNumber::Add,
                left: Box::new(Expr::Variable("x".to_string())),
                right: Box::new(Expr::Variable("x".to_string())),
            })),
        };
        let program = typed::Program {
            functions: vec![double],
            statements: vec![],
            exports: vec![Expr::Node(typed::Graphic::Circle {
                radius: Box::new(Expr::Call {
                    function: "double".to_string(),
                    arguments: vec![ExprGeneric::Number(Expr::Node(NodeNumber::Literal(7.0)))],
                }),
            })],
        };
        let scene = eval_program(program).unwrap();
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], scene::Graphic::Circle { radius: 14.0 });
    }
}
