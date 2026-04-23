//! Type resolver for expressions

use crate::ir::ast;
use crate::ir::typed::expr::{
    ExprBool, ExprColor, ExprGeneric, ExprGraphic, ExprNumber, ExprString, NodeBool, NodeNumber,
    OpBinNumber, OpCompare,
};
use crate::ir::typed::{self, Type};
use crate::type_checker::{Constraint, Context, TResult};
use Constraint::*;

pub fn check_generic(context: &Context, ty: Type, expr: ast::Expr) -> TResult<ExprGeneric> {
    match ty {
        Type::Number => Ok(ExprGeneric::Number(check_number(context, expr)?)),
        Type::String => Ok(ExprGeneric::String(check_string(context, expr)?)),
        Type::Bool => Ok(ExprGeneric::Bool(check_bool(context, expr)?)),
        Type::Color => Ok(ExprGeneric::Color(check_color(context, expr)?)),
        Type::Graphic => Ok(ExprGeneric::Graphic(check_graphic(context, expr)?)),
    }
}

fn check_func_args(
    context: &Context,
    function: &str,
    args: Vec<ast::Expr>,
    return_constraint: Constraint,
) -> TResult<Vec<ExprGeneric>> {
    let arg_types = context.check_function(&function, return_constraint)?;
    if arg_types.len() != args.len() {
        return Err("Invalid number of arguments".to_string());
    }
    args.into_iter()
        .zip(arg_types)
        .map(|(arg, ty)| check_generic(context, ty, arg))
        .collect::<Result<Vec<_>, _>>()
}

/// Checks an expression expecting a Number type.
pub fn check_number(context: &Context, expr: ast::Expr) -> TResult<ExprNumber> {
    use NodeNumber::{Binary, Literal};
    match expr {
        ast::Expr::LNumber(num) => Ok(ExprNumber::Node(Literal(num))),
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::Number))?;
            Ok(ExprNumber::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::Number))?;
            Ok(ExprNumber::Call {
                function,
                arguments: typed_args,
            })
        }
        ast::Expr::Binary {
            operator,
            left,
            right,
        } => {
            let op = map_op_num(operator).ok_or("Invalid operator for number")?;
            let left = check_number(context, *left)?;
            let right = check_number(context, *right)?;
            Ok(ExprNumber::Node(Binary {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
            }))
        }
        _ => Err("Unexpected expression, expected a number".to_string()),
    }
}

/// Maps general binary operators to number binary operations.
fn map_op_num(op: ast::OpBin) -> Option<OpBinNumber> {
    match op {
        ast::OpBin::Add => Some(OpBinNumber::Add),
        ast::OpBin::Sub => Some(OpBinNumber::Sub),
        ast::OpBin::Mul => Some(OpBinNumber::Mul),
        ast::OpBin::Div => Some(OpBinNumber::Div),
        _ => None,
    }
}

/// Maps general binary operators to comparison operations.
fn map_op_compare(op: ast::OpBin) -> Option<OpCompare> {
    match op {
        ast::OpBin::Gt => Some(OpCompare::Gt),
        ast::OpBin::Gte => Some(OpCompare::Gte),
        ast::OpBin::Lt => Some(OpCompare::Lt),
        ast::OpBin::Lte => Some(OpCompare::Lte),
        ast::OpBin::Eq => Some(OpCompare::Eq),
        ast::OpBin::Neq => Some(OpCompare::Neq),
        _ => None,
    }
}

/// Checks an expression expecting a Bool type.
pub fn check_bool(context: &Context, expr: ast::Expr) -> TResult<ExprBool> {
    use NodeBool::{Compare, Literal};
    match expr {
        ast::Expr::LBool(b) => Ok(ExprBool::Node(Literal(b))),
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::Bool))?;
            Ok(ExprBool::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::Bool))?;
            Ok(ExprBool::Call {
                function,
                arguments: typed_args,
            })
        }
        ast::Expr::Binary {
            operator,
            left,
            right,
        } => {
            let op = map_op_compare(operator).ok_or("Invalid operator for bool")?;
            let left = check_number(context, *left)?;
            let right = check_number(context, *right)?;
            Ok(ExprBool::Node(Compare {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
            }))
        }
        _ => Err("Unexpected expression, expected a bool".to_string()),
    }
}

/// Checks an expression expecting a String type.
pub fn check_string(context: &Context, expr: ast::Expr) -> TResult<ExprString> {
    match expr {
        ast::Expr::LString(s) => Ok(ExprString::Node(s)),
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::String))?;
            Ok(ExprString::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::String))?;
            Ok(ExprString::Call {
                function,
                arguments: typed_args,
            })
        }
        _ => Err("Unexpected expression, expected a string".to_string()),
    }
}

/// Checks an expression expecting a Color type.
pub fn check_color(context: &Context, expr: ast::Expr) -> TResult<ExprColor> {
    match expr {
        ast::Expr::LColor(ast::Color::Rgba { r, g, b, a }) => {
            let r = Box::new(check_number(context, *r)?);
            let g = Box::new(check_number(context, *g)?);
            let b = Box::new(check_number(context, *b)?);
            let a = Box::new(check_number(context, *a)?);
            Ok(ExprColor::Node(typed::Color::Rgba { r, g, b, a }))
        }
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::Color))?;
            Ok(ExprColor::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::Color))?;
            Ok(ExprColor::Call {
                function,
                arguments: typed_args,
            })
        }
        _ => Err("Unexpected expression, expected a color".to_string()),
    }
}

/// Checks an expression expecting a Graphic type.
pub fn check_graphic(context: &Context, expr: ast::Expr) -> TResult<ExprGraphic> {
    match expr {
        ast::Expr::LGraphic(ast::Graphic::Circle { radius }) => {
            let radius = Box::new(check_number(context, *radius)?);
            Ok(ExprGraphic::Node(typed::Graphic::Circle { radius }))
        }
        ast::Expr::LGraphic(ast::Graphic::Rect { width, height }) => {
            let width = Box::new(check_number(context, *width)?);
            let height = Box::new(check_number(context, *height)?);
            Ok(ExprGraphic::Node(typed::Graphic::Rect { width, height }))
        }
        ast::Expr::LGraphic(ast::Graphic::Text(text)) => {
            let text = Box::new(check_string(context, *text)?);
            Ok(ExprGraphic::Node(typed::Graphic::Text(text)))
        }
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::Graphic))?;
            Ok(ExprGraphic::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::Graphic))?;
            Ok(ExprGraphic::Call {
                function,
                arguments: typed_args,
            })
        }
        _ => Err("Unexpected expression, expected a graphic".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::OpBin;

    #[test]
    fn test_check_literal() {
        let context = Context::new();

        // Test LNumber
        let expr = ast::Expr::LNumber(10.0);
        let res = check_number(&context, expr).unwrap();
        assert_eq!(res, ExprNumber::Node(NodeNumber::Literal(10.0)));

        // Test check_number on a string (failure)
        let expr = ast::Expr::LString("foo".to_string());
        assert!(check_number(&context, expr).is_err());
    }

    #[test]
    fn test_check_variable() {
        let mut context = Context::new();
        context.set_var("x".to_string(), Type::Number);

        let expr = ast::Expr::Variable("x".to_string());
        let res = check_number(&context, expr.clone()).unwrap();
        assert_eq!(res, ExprNumber::Variable("x".to_string()));
        // check_string should fail for a number variable
        assert!(check_string(&context, expr).is_err());

        let expr = ast::Expr::Variable("y".to_string());
        assert!(check_number(&context, expr).is_err());
    }

    #[test]
    fn test_check_binary() {
        let mut context = Context::new();
        context.set_var("x".to_string(), Type::Number);

        // x + 10
        let expr = ast::Expr::Binary {
            operator: OpBin::Add,
            left: Box::new(ast::Expr::Variable("x".to_string())),
            right: Box::new(ast::Expr::LNumber(10.0)),
        };
        let res = check_number(&context, expr).unwrap();
        match res {
            ExprNumber::Node(NodeNumber::Binary { operator, .. }) => {
                assert_eq!(operator, OpBinNumber::Add);
            }
            _ => panic!("Expected binary node, got {:?}", res),
        }

        // Invalid: x + "foo"
        let expr = ast::Expr::Binary {
            operator: OpBin::Add,
            left: Box::new(ast::Expr::Variable("x".to_string())),
            right: Box::new(ast::Expr::LString("foo".to_string())),
        };
        assert!(check_number(&context, expr).is_err());
    }

    #[test]
    fn test_check_generic() {
        let context = Context::new();
        let expr = ast::Expr::LNumber(10.0);
        let res = check_generic(&context, Type::Number, expr).unwrap();
        assert!(matches!(res, ExprGeneric::Number(_)));

        let expr = ast::Expr::LString("foo".to_string());
        let res = check_generic(&context, Type::String, expr).unwrap();
        assert!(matches!(res, ExprGeneric::String(_)));

        // Type mismatch
        let expr = ast::Expr::LNumber(10.0);
        assert!(check_generic(&context, Type::String, expr).is_err());

        // Bool literal via generic
        let expr = ast::Expr::LBool(true);
        let res = check_generic(&context, Type::Bool, expr).unwrap();
        assert!(matches!(res, ExprGeneric::Bool(_)));
    }

    #[test]
    fn test_check_bool_literal_and_var() {
        let mut context = Context::new();
        context.set_var("b".to_string(), Type::Bool);

        let expr = ast::Expr::LBool(false);
        let res = check_bool(&context, expr).unwrap();
        assert_eq!(res, ExprBool::Node(NodeBool::Literal(false)));

        let expr = ast::Expr::Variable("b".to_string());
        let res = check_bool(&context, expr).unwrap();
        assert_eq!(res, ExprBool::Variable("b".to_string()));

        // Wrong-typed variable
        let expr = ast::Expr::Variable("missing".to_string());
        assert!(check_bool(&context, expr).is_err());
    }

    #[test]
    fn test_check_bool_compare() {
        let context = Context::new();

        // 1 > 2
        let expr = ast::Expr::Binary {
            operator: OpBin::Gt,
            left: Box::new(ast::Expr::LNumber(1.0)),
            right: Box::new(ast::Expr::LNumber(2.0)),
        };
        let res = check_bool(&context, expr).unwrap();
        match res {
            ExprBool::Node(NodeBool::Compare { operator, .. }) => {
                assert_eq!(operator, OpCompare::Gt);
            }
            _ => panic!("Expected compare, got {:?}", res),
        }

        // Comparison with non-number operand fails
        let expr = ast::Expr::Binary {
            operator: OpBin::Gt,
            left: Box::new(ast::Expr::LBool(true)),
            right: Box::new(ast::Expr::LNumber(2.0)),
        };
        assert!(check_bool(&context, expr).is_err());
    }

    #[test]
    fn test_check_number_rejects_compare() {
        let context = Context::new();
        // 1 > 2 cannot satisfy a Number context
        let expr = ast::Expr::Binary {
            operator: OpBin::Gt,
            left: Box::new(ast::Expr::LNumber(1.0)),
            right: Box::new(ast::Expr::LNumber(2.0)),
        };
        assert!(check_number(&context, expr).is_err());
    }

    #[test]
    fn test_check_bool_rejects_arithmetic() {
        let context = Context::new();
        // 1 + 2 cannot satisfy a Bool context
        let expr = ast::Expr::Binary {
            operator: OpBin::Add,
            left: Box::new(ast::Expr::LNumber(1.0)),
            right: Box::new(ast::Expr::LNumber(2.0)),
        };
        assert!(check_bool(&context, expr).is_err());
    }
}
