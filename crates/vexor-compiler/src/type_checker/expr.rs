//! Type resolver for expressions

use crate::ir::ast::{self, OpBin};
use crate::ir::typed::expr::{
    ExprColor, ExprGeneric, ExprGraphic, ExprNumber, ExprString, NodeNumber, OpBinNumber,
};
use crate::ir::typed::{self, Type};
use crate::type_checker::{Constraint, Context, TResult};
use Constraint::*;

pub fn check_generic(context: &Context, ty: Type, expr: ast::Expr) -> TResult<ExprGeneric> {
    match ty {
        Type::Number => Ok(ExprGeneric::Number(check_number(context, expr)?)),
        Type::String => Ok(ExprGeneric::String(check_string(context, expr)?)),
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
            check_op(Type::Number, operator)?;
            let left = check_number(context, *left)?;
            let right = check_number(context, *right)?;
            Ok(ExprNumber::Node(Binary {
                operator: map_op_num(operator),
                left: Box::new(left),
                right: Box::new(right),
            }))
        }
        _ => Err("Unexpected expression, expected a number".to_string()),
    }
}

/// Maps general binary operators to number binary operations.
fn map_op_num(op: ast::OpBin) -> OpBinNumber {
    match op {
        ast::OpBin::Add => OpBinNumber::Add,
        ast::OpBin::Sub => OpBinNumber::Sub,
        ast::OpBin::Mul => OpBinNumber::Mul,
        ast::OpBin::Div => OpBinNumber::Div,
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

/// Checks that an operator is valid for a given type.
fn check_op(ty: Type, op: OpBin) -> TResult<()> {
    match ty {
        Type::Number => match op {
            OpBin::Add | OpBin::Sub | OpBin::Mul | OpBin::Div => Ok(()),
        },
        _ => Err("Invalid operator for type".to_string()),
    }
}

// Not needed for now, operators have homogeneous types
// /// Determine constraint of last operand in binary expression.
// fn binary_constraint(op: OpBin, left: Type) -> TResult<Constraint> {
//     match left {
//         Type::Number => match op {
//             OpBin::Add | OpBin::Sub | OpBin::Mul | OpBin::Div => Ok(Is(Type::Number)),
//         },
//         _ => Err("Invalid operator for type".to_string()),
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
