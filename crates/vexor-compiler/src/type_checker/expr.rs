//! Type resolver for expressions

use crate::ir::ast::{self, OpBin};
use crate::ir::typed::{Type, ast as typed_ast};
use crate::type_checker::{Constraint, Context, TResult, check_identifier};
use Constraint::*;

pub fn check_expr(
    context: &Context,
    expr: ast::Expr,
    constraint: Constraint,
) -> TResult<typed_ast::Expr> {
    match expr {
        ast::Expr::LNumber(_)
        | ast::Expr::LString(_)
        | ast::Expr::LColor(_)
        | ast::Expr::LGraphic(_) => check_literal(expr, constraint),
        ast::Expr::Variable(ref id) => {
            check_identifier(context, id, constraint).map(|ty| typed_ast::Expr { expr, ty })
        }
        ast::Expr::Binary {
            operator,
            left,
            right,
        } => {
            let left = check_expr(context, *left, Any)?;
            let constraint_r = binary_constraint(operator, left.ty)?;
            let right = check_expr(context, *right, constraint_r)?;
            // Assume binary operator returns the type of the left operand
            left.ty.satisfies(constraint).map(|ty| typed_ast::Expr {
                expr: ast::Expr::Binary {
                    operator,
                    left: Box::new(left.expr),
                    right: Box::new(right.expr),
                },
                ty,
            })
        }
    }
}

/// Resolves a literal expression to a typed expression.
///   Must only call for literal expressions.
fn check_literal(expr: ast::Expr, constraint: Constraint) -> TResult<typed_ast::Expr> {
    let ty = match expr {
        ast::Expr::LNumber(_) => Type::Number,
        ast::Expr::LString(_) => Type::String,
        ast::Expr::LColor(_) => Type::Color,
        ast::Expr::LGraphic(_) => Type::Graphic,
        _ => unreachable!(),
    };
    ty.satisfies(constraint)
        .map(|ty| typed_ast::Expr { expr, ty })
}

/// Determine constraint of last operand in binary expression.
fn binary_constraint(op: OpBin, left: Type) -> TResult<Constraint> {
    match left {
        Type::Number => match op {
            OpBin::Add | OpBin::Sub | OpBin::Mul | OpBin::Div => Ok(Is(Type::Number)),
        },
        _ => Err("Invalid operator for type".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_check_literal() {
        let context = Context {
            var_types: HashMap::new(),
        };

        // Test LNumber
        let expr = ast::Expr::LNumber(10.0);
        let res = check_expr(&context, expr.clone(), Any).unwrap();
        assert_eq!(res.ty, Type::Number);
        assert_eq!(res.expr, expr);

        // Test LString with constraint mismatch
        let expr = ast::Expr::LString("foo".to_string());
        let res = check_expr(&context, expr, Is(Type::Number));
        assert!(res.is_err());
    }

    #[test]
    fn test_check_variable() {
        let mut var_types = HashMap::new();
        var_types.insert("x".to_string(), Type::Number);
        let context = Context { var_types };

        let expr = ast::Expr::Variable("x".to_string());
        let res = check_expr(&context, expr.clone(), Is(Type::Number)).unwrap();
        assert_eq!(res.ty, Type::Number);
        assert_eq!(res.expr, expr);

        let expr = ast::Expr::Variable("y".to_string());
        assert!(check_expr(&context, expr, Any).is_err());
    }

    #[test]
    fn test_check_binary() {
        let mut var_types = HashMap::new();
        var_types.insert("x".to_string(), Type::Number);
        let context = Context { var_types };

        // x + 10
        let expr = ast::Expr::Binary {
            operator: OpBin::Add,
            left: Box::new(ast::Expr::Variable("x".to_string())),
            right: Box::new(ast::Expr::LNumber(10.0)),
        };
        let res = check_expr(&context, expr, Is(Type::Number)).unwrap();
        assert_eq!(res.ty, Type::Number);

        // Invalid: x + "foo"
        let expr = ast::Expr::Binary {
            operator: OpBin::Add,
            left: Box::new(ast::Expr::Variable("x".to_string())),
            right: Box::new(ast::Expr::LString("foo".to_string())),
        };
        assert!(check_expr(&context, expr, Any).is_err());

        // Invalid operator for type: "foo" + 1
        let expr = ast::Expr::Binary {
            operator: OpBin::Add,
            left: Box::new(ast::Expr::LString("foo".to_string())),
            right: Box::new(ast::Expr::LNumber(1.0)),
        };
        assert!(check_expr(&context, expr, Any).is_err());
    }
}
