//! Type resolver for expressions

use crate::ir::ast;
use crate::ir::typed::expr::{
    Expr, ExprBool, ExprColor, ExprGeneric, ExprGraphic, ExprNumber, ExprString, MatchArm,
    NodeBool, NodeColor, NodeGraphic, NodeNumber, NodeString, OpBinBool, OpBinNumber, OpCompare,
    OpUnBool, Pattern,
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
        Type::GType(_) => Ok(ExprGeneric::Graphic(check_graphic(context, expr)?)),
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

/// Type-checks the condition (bool) and both branches (of type E) of an if expression.
fn check_if<F, E>(
    context: &Context,
    condition: ast::Expr,
    then_branch: ast::Expr,
    else_branch: ast::Expr,
    check: F,
) -> TResult<Expr<E>>
where
    F: Fn(&Context, ast::Expr) -> TResult<Expr<E>>,
{
    let condition = Box::new(check_bool(context, condition)?);
    let then_branch = Box::new(check(context, then_branch)?);
    let else_branch = Box::new(check(context, else_branch)?);
    Ok(Expr::If {
        condition,
        then_branch,
        else_branch,
    })
}

/// Type-checks match arms for a match whose scrutinee and body are of type E.
fn check_match_arms<F, E>(
    context: &Context,
    ty: Type,
    arms: Vec<ast::MatchArm>,
    check: F,
) -> TResult<Vec<MatchArm<E>>>
where
    F: Fn(&Context, ast::Expr) -> TResult<E>,
{
    arms.into_iter()
        .map(|arm| {
            let ast::MatchArm {
                pattern,
                guard,
                body,
            } = arm;
            let (pattern, scope) = match pattern {
                ast::Pattern::Binding(name) => {
                    let scope = context.with_var(name.clone(), ty);
                    (Pattern::Binding(name), Some(scope))
                }
                ast::Pattern::Literal(e) => (Pattern::Literal(check(context, e)?), None),
            };
            let arm_ctx = scope.as_ref().unwrap_or(context);
            let guard = guard.map(|g| check_bool(arm_ctx, g)).transpose()?;
            let body = check(arm_ctx, body)?;
            Ok(MatchArm {
                pattern,
                guard,
                body,
            })
        })
        .collect()
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

/// Maps general binary operators to bool binary operations.
fn map_op_bool(op: ast::OpBin) -> Option<OpBinBool> {
    match op {
        ast::OpBin::And => Some(OpBinBool::And),
        ast::OpBin::Or => Some(OpBinBool::Or),
        ast::OpBin::Eq => Some(OpBinBool::Eq),
        ast::OpBin::Neq => Some(OpBinBool::Neq),
        _ => None,
    }
}

/// Checks an expression expecting a Number type.
pub fn check_number(context: &Context, expr: ast::Expr) -> TResult<ExprNumber> {
    use NodeNumber::{Binary, Literal, Match};
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
        ast::Expr::Match { scrutinee, arms } => {
            let scrutinee = Box::new(check_number(context, *scrutinee)?);
            let arms = check_match_arms(context, Type::Number, arms, check_number)?;
            Ok(ExprNumber::Node(Match { scrutinee, arms }))
        }
        ast::Expr::If {
            condition,
            then_branch,
            else_branch,
        } => check_if(
            context,
            *condition,
            *then_branch,
            *else_branch,
            check_number,
        ),
        ast::Expr::Field { object, field } => {
            check_field_access(context, object, field, Is(Type::Number))
        }
        ast::Expr::Unary { .. }
        | ast::Expr::LBool(_)
        | ast::Expr::LString(_)
        | ast::Expr::LColor(_)
        | ast::Expr::LObject(_) => Err("Unexpected expression, expected a number".to_string()),
    }
}

/// Checks an expression expecting a Bool type.
pub fn check_bool(context: &Context, expr: ast::Expr) -> TResult<ExprBool> {
    use NodeBool::{Binary, Compare, Literal, Match, Unary};
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
        ast::Expr::Unary {
            operator: ast::OpUn::Not,
            operand,
        } => {
            let operand = check_bool(context, *operand)?;
            Ok(ExprBool::Node(Unary {
                operator: OpUnBool::Not,
                operand: Box::new(operand),
            }))
        }
        ast::Expr::Binary {
            operator,
            left,
            right,
        } => {
            // Try bool-binary first (Fall back if Eq/Neq to number compare).
            if let Some(op) = map_op_bool(operator) {
                let l_bool = check_bool(context, (*left).clone());
                let r_bool = check_bool(context, (*right).clone());
                if let (Ok(l), Ok(r)) = (l_bool, r_bool) {
                    return Ok(ExprBool::Node(Binary {
                        operator: op,
                        left: Box::new(l),
                        right: Box::new(r),
                    }));
                }
                // Reject if cannot fallback to number compare
                // only Eq/Neq can fallback to number compare
                if !matches!(op, OpBinBool::Eq | OpBinBool::Neq) {
                    return Err("Logical operator requires bool operands".to_string());
                }
            }
            let op = map_op_compare(operator).ok_or("Invalid operator for bool")?;
            let left = check_number(context, *left)?;
            let right = check_number(context, *right)?;
            Ok(ExprBool::Node(Compare {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
            }))
        }
        ast::Expr::Match { scrutinee, arms } => {
            let scrutinee = Box::new(check_bool(context, *scrutinee)?);
            let arms = check_match_arms(context, Type::Bool, arms, check_bool)?;
            Ok(ExprBool::Node(Match { scrutinee, arms }))
        }
        ast::Expr::If {
            condition,
            then_branch,
            else_branch,
        } => check_if(context, *condition, *then_branch, *else_branch, check_bool),
        ast::Expr::Field { object, field } => {
            check_field_access(context, object, field, Is(Type::Bool))
        }
        ast::Expr::LNumber(_)
        | ast::Expr::LString(_)
        | ast::Expr::LColor(_)
        | ast::Expr::LObject(_) => Err("Unexpected expression, expected a bool".to_string()),
    }
}

/// Checks an expression expecting a String type.
pub fn check_string(context: &Context, expr: ast::Expr) -> TResult<ExprString> {
    match expr {
        ast::Expr::LString(s) => Ok(ExprString::Node(NodeString::Literal(s))),
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
        ast::Expr::Match { scrutinee, arms } => {
            let scrutinee = Box::new(check_string(context, *scrutinee)?);
            let arms = check_match_arms(context, Type::String, arms, check_string)?;
            Ok(ExprString::Node(NodeString::Match { scrutinee, arms }))
        }
        ast::Expr::If {
            condition,
            then_branch,
            else_branch,
        } => check_if(
            context,
            *condition,
            *then_branch,
            *else_branch,
            check_string,
        ),
        ast::Expr::Field { object, field } => {
            check_field_access(context, object, field, Is(Type::String))
        }
        ast::Expr::Binary { .. }
        | ast::Expr::Unary { .. }
        | ast::Expr::LNumber(_)
        | ast::Expr::LBool(_)
        | ast::Expr::LColor(_)
        | ast::Expr::LObject(_) => Err("Unexpected expression, expected a string".to_string()),
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
            Ok(ExprColor::Node(NodeColor::Literal(typed::Color::Rgba {
                r,
                g,
                b,
                a,
            })))
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
        ast::Expr::Match { scrutinee, arms } => {
            let scrutinee = Box::new(check_color(context, *scrutinee)?);
            let arms = check_match_arms(context, Type::Color, arms, check_color)?;
            Ok(ExprColor::Node(NodeColor::Match { scrutinee, arms }))
        }
        ast::Expr::If {
            condition,
            then_branch,
            else_branch,
        } => check_if(context, *condition, *then_branch, *else_branch, check_color),
        ast::Expr::Field { object, field } => {
            check_field_access(context, object, field, Is(Type::Color))
        }
        ast::Expr::Binary { .. }
        | ast::Expr::Unary { .. }
        | ast::Expr::LNumber(_)
        | ast::Expr::LBool(_)
        | ast::Expr::LString(_)
        | ast::Expr::LObject(_) => Err("Unexpected expression, expected a color".to_string()),
    }
}

macro_rules! extract_fields {
    ($fields:expr, [$($name:ident),+]) => {{
        let fields: Vec<(String, _)> = $fields;

        // Check for unexpected
        let expected = [$(stringify!($name)),+];
        for (key, _) in fields.iter() {
            if !expected.contains(&key.as_str()) {
                // TODO: Return multiple errors for each unexpected field
                return Err(format!("unexpected field: {key}"));
            }
        }

        // Extract and move fields to tupl
        let mut map: std::collections::HashMap<String, _> = fields.into_iter().collect();

        $(
            let $name = map.remove(stringify!($name))
                .ok_or_else(|| format!("missing field: {}", stringify!($name)))?;
        )+

        ($($name),+)
    }};
}

/// Checks a field access type
fn check_field_access<T>(
    context: &Context,
    object: String,
    field: String,
    constraint: Constraint,
) -> TResult<Expr<T>> {
    // Only Graphic types can have fields, for now
    let obj_ty = context.check_var(&object, Is(Type::Graphic))?;
    let Type::GType(obj_ty) = obj_ty else {
        return Err("Expected Graphic type".to_string());
    };
    let field_ty = match obj_ty {
        typed::GraphicType::Circle => match field.as_str() {
            "x" | "y" | "radius" => Type::Number,
            "color" => Type::Color,
            _ => return Err(format!("Unknown field {field} for Circle type")),
        },
        typed::GraphicType::Rect => match field.as_str() {
            "x" | "y" | "width" | "height" => Type::Number,
            "color" => Type::Color,
            _ => return Err(format!("Unknown field {field} for Rect type")),
        },
        typed::GraphicType::Text => match field.as_str() {
            "x" | "y" => Type::Number,
            "content" => Type::String,
            "color" => Type::Color,
            _ => return Err(format!("Unknown field {field} for Text type")),
        },
    };
    field_ty
        .satisfies(constraint)
        .map(|_| Expr::Field { object, field })
}

/// Checks an expression expecting a Graphic type.
pub fn check_graphic(context: &Context, expr: ast::Expr) -> TResult<ExprGraphic> {
    match expr {
        ast::Expr::LObject(ast::Object { name, fields }) => match name.as_str() {
            "Circle" => {
                let (x, y, radius, color) = extract_fields!(fields, [x, y, radius, color]);
                Ok(ExprGraphic::Node(NodeGraphic::Literal(
                    typed::Graphic::Circle {
                        x: Box::new(check_number(context, x)?),
                        y: Box::new(check_number(context, y)?),
                        radius: Box::new(check_number(context, radius)?),
                        color: Box::new(check_color(context, color)?),
                    },
                )))
            }
            "Rect" => {
                let (x, y, width, height, color) =
                    extract_fields!(fields, [x, y, width, height, color]);
                Ok(ExprGraphic::Node(NodeGraphic::Literal(
                    typed::Graphic::Rect {
                        x: Box::new(check_number(context, x)?),
                        y: Box::new(check_number(context, y)?),
                        width: Box::new(check_number(context, width)?),
                        height: Box::new(check_number(context, height)?),
                        color: Box::new(check_color(context, color)?),
                    },
                )))
            }
            "Text" => {
                let (x, y, content, color) = extract_fields!(fields, [x, y, content, color]);
                Ok(ExprGraphic::Node(NodeGraphic::Literal(
                    typed::Graphic::Text {
                        x: Box::new(check_number(context, x)?),
                        y: Box::new(check_number(context, y)?),
                        content: Box::new(check_string(context, content)?),
                        color: Box::new(check_color(context, color)?),
                    },
                )))
            }

            _ => Err("Unexpected object name".to_string()),
        },
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
        ast::Expr::Match { scrutinee, arms } => {
            let scrutinee = Box::new(check_graphic(context, *scrutinee)?);
            let arms = check_match_arms(context, Type::Graphic, arms, check_graphic)?;
            Ok(ExprGraphic::Node(NodeGraphic::Match { scrutinee, arms }))
        }
        ast::Expr::If {
            condition,
            then_branch,
            else_branch,
        } => check_if(
            context,
            *condition,
            *then_branch,
            *else_branch,
            check_graphic,
        ),
        ast::Expr::Field { object, field } => {
            check_field_access(context, object, field, Is(Type::Graphic))
        }
        ast::Expr::Binary { .. }
        | ast::Expr::Unary { .. }
        | ast::Expr::LNumber(_)
        | ast::Expr::LBool(_)
        | ast::Expr::LString(_)
        | ast::Expr::LColor(_) => Err("Unexpected expression, expected a graphic".to_string()),
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

    #[test]
    fn test_check_bool_logical_binary() {
        let context = Context::new();
        // true && false
        let expr = ast::Expr::Binary {
            operator: OpBin::And,
            left: Box::new(ast::Expr::LBool(true)),
            right: Box::new(ast::Expr::LBool(false)),
        };
        let res = check_bool(&context, expr).unwrap();
        match res {
            ExprBool::Node(NodeBool::Binary { operator, .. }) => {
                assert_eq!(operator, OpBinBool::And);
            }
            _ => panic!("Expected Binary And, got {:?}", res),
        }

        // true || false
        let expr = ast::Expr::Binary {
            operator: OpBin::Or,
            left: Box::new(ast::Expr::LBool(true)),
            right: Box::new(ast::Expr::LBool(false)),
        };
        let res = check_bool(&context, expr).unwrap();
        match res {
            ExprBool::Node(NodeBool::Binary { operator, .. }) => {
                assert_eq!(operator, OpBinBool::Or);
            }
            _ => panic!("Expected Binary Or, got {:?}", res),
        }
    }

    #[test]
    fn test_check_bool_not() {
        let context = Context::new();
        let expr = ast::Expr::Unary {
            operator: ast::OpUn::Not,
            operand: Box::new(ast::Expr::LBool(true)),
        };
        let res = check_bool(&context, expr).unwrap();
        match res {
            ExprBool::Node(NodeBool::Unary { operator, .. }) => {
                assert_eq!(operator, OpUnBool::Not);
            }
            _ => panic!("Expected Unary Not, got {:?}", res),
        }

        // !1 rejected
        let expr = ast::Expr::Unary {
            operator: ast::OpUn::Not,
            operand: Box::new(ast::Expr::LNumber(1.0)),
        };
        assert!(check_bool(&context, expr).is_err());
    }

    #[test]
    fn test_check_bool_logical_rejects_non_bool() {
        let context = Context::new();
        // 1 && 2 — And requires bool operands
        let expr = ast::Expr::Binary {
            operator: OpBin::And,
            left: Box::new(ast::Expr::LNumber(1.0)),
            right: Box::new(ast::Expr::LNumber(2.0)),
        };
        assert!(check_bool(&context, expr).is_err());
    }

    #[test]
    fn test_check_match_basic() {
        let mut context = Context::new();
        context.set_var("x".to_string(), Type::Number);

        // match x { x if x > 10 => 100, 2 => 99, y => y + 1 }
        let expr = ast::Expr::Match {
            scrutinee: Box::new(ast::Expr::Variable("x".to_string())),
            arms: vec![
                ast::MatchArm {
                    pattern: ast::Pattern::Binding("x".to_string()),
                    guard: Some(ast::Expr::Binary {
                        operator: OpBin::Gt,
                        left: Box::new(ast::Expr::Variable("x".to_string())),
                        right: Box::new(ast::Expr::LNumber(10.0)),
                    }),
                    body: ast::Expr::LNumber(100.0),
                },
                ast::MatchArm {
                    pattern: ast::Pattern::Literal(ast::Expr::LNumber(2.0)),
                    guard: None,
                    body: ast::Expr::LNumber(99.0),
                },
                ast::MatchArm {
                    pattern: ast::Pattern::Binding("y".to_string()),
                    guard: None,
                    body: ast::Expr::Binary {
                        operator: OpBin::Add,
                        left: Box::new(ast::Expr::Variable("y".to_string())),
                        right: Box::new(ast::Expr::LNumber(1.0)),
                    },
                },
            ],
        };
        let res = check_number(&context, expr).unwrap();
        match res {
            ExprNumber::Node(NodeNumber::Match { arms, .. }) => {
                assert_eq!(arms.len(), 3);
            }
            _ => panic!("Expected Match node, got {:?}", res),
        }
    }

    #[test]
    fn test_check_match_non_bool_guard() {
        let context = Context::new();
        // match 1 { x if x => 0 }  — guard is number, not bool.
        let expr = ast::Expr::Match {
            scrutinee: Box::new(ast::Expr::LNumber(1.0)),
            arms: vec![ast::MatchArm {
                pattern: ast::Pattern::Binding("x".to_string()),
                guard: Some(ast::Expr::Variable("x".to_string())),
                body: ast::Expr::LNumber(0.0),
            }],
        };
        assert!(check_number(&context, expr).is_err());
    }

    #[test]
    fn test_check_match_non_number_body() {
        let context = Context::new();
        // match 1 { x => "foo" } — body is string, context expects number.
        let expr = ast::Expr::Match {
            scrutinee: Box::new(ast::Expr::LNumber(1.0)),
            arms: vec![ast::MatchArm {
                pattern: ast::Pattern::Binding("x".to_string()),
                guard: None,
                body: ast::Expr::LString("foo".to_string()),
            }],
        };
        assert!(check_number(&context, expr).is_err());
    }

    #[test]
    fn test_check_match_string() {
        let context = Context::new();
        // match "a" { "a" => "yes", x => x }
        let expr = ast::Expr::Match {
            scrutinee: Box::new(ast::Expr::LString("a".to_string())),
            arms: vec![
                ast::MatchArm {
                    pattern: ast::Pattern::Literal(ast::Expr::LString("a".to_string())),
                    guard: None,
                    body: ast::Expr::LString("yes".to_string()),
                },
                ast::MatchArm {
                    pattern: ast::Pattern::Binding("x".to_string()),
                    guard: None,
                    body: ast::Expr::Variable("x".to_string()),
                },
            ],
        };
        let res = check_string(&context, expr).unwrap();
        assert!(matches!(res, ExprString::Node(NodeString::Match { .. })));
    }

    #[test]
    fn test_check_match_bool() {
        let context = Context::new();
        // match true { true => false, x => x }
        let expr = ast::Expr::Match {
            scrutinee: Box::new(ast::Expr::LBool(true)),
            arms: vec![
                ast::MatchArm {
                    pattern: ast::Pattern::Literal(ast::Expr::LBool(true)),
                    guard: None,
                    body: ast::Expr::LBool(false),
                },
                ast::MatchArm {
                    pattern: ast::Pattern::Binding("x".to_string()),
                    guard: None,
                    body: ast::Expr::Variable("x".to_string()),
                },
            ],
        };
        let res = check_bool(&context, expr).unwrap();
        assert!(matches!(res, ExprBool::Node(NodeBool::Match { .. })));
    }

    #[test]
    fn test_check_if_number() {
        let context = Context::new();
        // if true { 1 } else { 2 }  in number context
        let expr = ast::Expr::If {
            condition: Box::new(ast::Expr::LBool(true)),
            then_branch: Box::new(ast::Expr::LNumber(1.0)),
            else_branch: Box::new(ast::Expr::LNumber(2.0)),
        };
        let res = check_number(&context, expr).unwrap();
        assert!(matches!(res, Expr::If { .. }));
    }

    #[test]
    fn test_check_if_string() {
        let context = Context::new();
        let expr = ast::Expr::If {
            condition: Box::new(ast::Expr::LBool(true)),
            then_branch: Box::new(ast::Expr::LString("a".to_string())),
            else_branch: Box::new(ast::Expr::LString("b".to_string())),
        };
        let res = check_string(&context, expr).unwrap();
        assert!(matches!(res, Expr::If { .. }));
    }

    #[test]
    fn test_check_if_bool() {
        let context = Context::new();
        let expr = ast::Expr::If {
            condition: Box::new(ast::Expr::LBool(true)),
            then_branch: Box::new(ast::Expr::LBool(false)),
            else_branch: Box::new(ast::Expr::LBool(true)),
        };
        let res = check_bool(&context, expr).unwrap();
        assert!(matches!(res, Expr::If { .. }));
    }

    #[test]
    fn test_check_if_non_bool_condition() {
        let context = Context::new();
        // if 1 { 1 } else { 2 }  — condition is number, not bool.
        let expr = ast::Expr::If {
            condition: Box::new(ast::Expr::LNumber(1.0)),
            then_branch: Box::new(ast::Expr::LNumber(1.0)),
            else_branch: Box::new(ast::Expr::LNumber(2.0)),
        };
        assert!(check_number(&context, expr).is_err());
    }

    #[test]
    fn test_check_if_branch_type_mismatch() {
        let context = Context::new();
        // number context, else branch is a string
        let expr = ast::Expr::If {
            condition: Box::new(ast::Expr::LBool(true)),
            then_branch: Box::new(ast::Expr::LNumber(1.0)),
            else_branch: Box::new(ast::Expr::LString("nope".to_string())),
        };
        assert!(check_number(&context, expr).is_err());
    }

    #[test]
    fn test_check_bool_eq_dispatch() {
        let context = Context::new();

        // bool == bool → Binary Eq
        let expr = ast::Expr::Binary {
            operator: OpBin::Eq,
            left: Box::new(ast::Expr::LBool(true)),
            right: Box::new(ast::Expr::LBool(false)),
        };
        let res = check_bool(&context, expr).unwrap();
        match res {
            ExprBool::Node(NodeBool::Binary { operator, .. }) => {
                assert_eq!(operator, OpBinBool::Eq);
            }
            _ => panic!("Expected Binary Eq for bool operands, got {:?}", res),
        }

        // number == number still → Compare Eq
        let expr = ast::Expr::Binary {
            operator: OpBin::Eq,
            left: Box::new(ast::Expr::LNumber(1.0)),
            right: Box::new(ast::Expr::LNumber(2.0)),
        };
        let res = check_bool(&context, expr).unwrap();
        match res {
            ExprBool::Node(NodeBool::Compare { operator, .. }) => {
                assert_eq!(operator, OpCompare::Eq);
            }
            _ => panic!("Expected Compare Eq for number operands, got {:?}", res),
        }
    }

    // --- if/match in Color/Graphic context ---

    fn red_literal() -> ast::Expr {
        ast::Expr::LColor(ast::Color::Rgba {
            r: Box::new(ast::Expr::LNumber(1.0)),
            g: Box::new(ast::Expr::LNumber(0.0)),
            b: Box::new(ast::Expr::LNumber(0.0)),
            a: Box::new(ast::Expr::LNumber(1.0)),
        })
    }

    fn blue_literal() -> ast::Expr {
        ast::Expr::LColor(ast::Color::Rgba {
            r: Box::new(ast::Expr::LNumber(0.0)),
            g: Box::new(ast::Expr::LNumber(0.0)),
            b: Box::new(ast::Expr::LNumber(1.0)),
            a: Box::new(ast::Expr::LNumber(1.0)),
        })
    }

    #[test]
    fn test_check_if_color() {
        let context = Context::new();
        let expr = ast::Expr::If {
            condition: Box::new(ast::Expr::LBool(true)),
            then_branch: Box::new(red_literal()),
            else_branch: Box::new(blue_literal()),
        };
        let res = check_color(&context, expr).unwrap();
        assert!(matches!(res, Expr::If { .. }));
    }

    #[test]
    fn test_check_if_color_wrong_branch_type() {
        let context = Context::new();
        // color context, else branch is a number
        let expr = ast::Expr::If {
            condition: Box::new(ast::Expr::LBool(true)),
            then_branch: Box::new(red_literal()),
            else_branch: Box::new(ast::Expr::LNumber(1.0)),
        };
        assert!(check_color(&context, expr).is_err());
    }

    #[test]
    fn test_check_match_color() {
        let context = Context::new();
        // match red { red => blue, x => x }
        let expr = ast::Expr::Match {
            scrutinee: Box::new(red_literal()),
            arms: vec![
                ast::MatchArm {
                    pattern: ast::Pattern::Literal(red_literal()),
                    guard: None,
                    body: blue_literal(),
                },
                ast::MatchArm {
                    pattern: ast::Pattern::Binding("x".to_string()),
                    guard: None,
                    body: ast::Expr::Variable("x".to_string()),
                },
            ],
        };
        let res = check_color(&context, expr).unwrap();
        assert!(matches!(res, ExprColor::Node(NodeColor::Match { .. })));
    }

    fn circle_literal() -> ast::Expr {
        ast::Expr::LObject(ast::Object {
            name: "Circle".to_string(),
            fields: vec![
                ("x".to_string(), ast::Expr::LNumber(0.0)),
                ("y".to_string(), ast::Expr::LNumber(0.0)),
                ("radius".to_string(), ast::Expr::LNumber(10.0)),
                ("color".to_string(), red_literal()),
            ],
        })
    }

    fn rect_literal() -> ast::Expr {
        ast::Expr::LObject(ast::Object {
            name: "Rect".to_string(),
            fields: vec![
                ("x".to_string(), ast::Expr::LNumber(0.0)),
                ("y".to_string(), ast::Expr::LNumber(0.0)),
                ("width".to_string(), ast::Expr::LNumber(5.0)),
                ("height".to_string(), ast::Expr::LNumber(5.0)),
                ("color".to_string(), blue_literal()),
            ],
        })
    }

    #[test]
    fn test_check_if_graphic() {
        let context = Context::new();
        let expr = ast::Expr::If {
            condition: Box::new(ast::Expr::LBool(true)),
            then_branch: Box::new(circle_literal()),
            else_branch: Box::new(rect_literal()),
        };
        let res = check_graphic(&context, expr).unwrap();
        assert!(matches!(res, Expr::If { .. }));
    }

    #[test]
    fn test_check_match_graphic() {
        let context = Context::new();
        // match circle { g => rect }
        let expr = ast::Expr::Match {
            scrutinee: Box::new(circle_literal()),
            arms: vec![ast::MatchArm {
                pattern: ast::Pattern::Binding("g".to_string()),
                guard: None,
                body: rect_literal(),
            }],
        };
        let res = check_graphic(&context, expr).unwrap();
        assert!(matches!(res, ExprGraphic::Node(NodeGraphic::Match { .. })));
    }

    #[test]
    fn test_check_field_access() {
        let mut context = Context::new();
        context.set_var("box".to_string(), Type::GType(typed::GraphicType::Rect));
        let expr = ast::Expr::Field {
            object: "box".to_string(),
            field: "color".to_string(),
        };
        let res = check_color(&context, expr).unwrap();
        assert!(matches!(res, ExprColor::Field { .. }));

        let expr = ast::Expr::Field {
            object: "box".to_string(),
            field: "width".to_string(),
        };
        let res = check_number(&context, expr).unwrap();
        assert!(matches!(res, ExprNumber::Field { .. }));

        let expr = ast::Expr::Field {
            object: "box".to_string(),
            field: "not_a_field".to_string(),
        };
        let res = check_number(&context, expr);
        assert!(matches!(res, Err(_)));
    }
}
