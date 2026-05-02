//! Type resolver for expressions

use crate::ir::ast;
use crate::ir::typed::expr::{
    ArithmeticOp, BoolOps, CompareOp, Expr, ExprGeneric, LogicOp, MatchArm, NumberOps, Pattern,
    SemanticType,
};
use crate::ir::typed::{self, BoolT, ColorT, GraphicT, NumberT, StringT, Type};
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
fn check_if<F, E: SemanticType>(
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
                ast::Pattern::Literal(e) => (
                    Pattern::Literal(check(context, ast::Expr::Literal(e))?),
                    None,
                ),
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

/// Checks a match expression.
fn check_match<F, E: SemanticType>(
    context: &Context,
    scrutinee: ast::Expr,
    arms: Vec<ast::MatchArm>,
    ty: Type,
    check: F,
) -> TResult<Expr<E>>
where
    F: Fn(&Context, ast::Expr) -> TResult<Expr<E>>,
{
    let scrutinee = Box::new(check(context, scrutinee)?);
    let arms = check_match_arms(context, ty, arms, check)?;
    Ok(Expr::Match { scrutinee, arms })
}

/// Maps general binary operators to number binary operations.
fn map_op_arithmetic(op: ast::OpBin) -> Option<ArithmeticOp> {
    match op {
        ast::OpBin::Add => Some(ArithmeticOp::Add),
        ast::OpBin::Sub => Some(ArithmeticOp::Sub),
        ast::OpBin::Mul => Some(ArithmeticOp::Mul),
        ast::OpBin::Div => Some(ArithmeticOp::Div),
        _ => None,
    }
}

/// Maps general binary operators to comparison operations.
fn map_op_compare(op: ast::OpBin) -> Option<CompareOp> {
    match op {
        ast::OpBin::Gt => Some(CompareOp::Gt),
        ast::OpBin::Gte => Some(CompareOp::Gte),
        ast::OpBin::Lt => Some(CompareOp::Lt),
        ast::OpBin::Lte => Some(CompareOp::Lte),
        ast::OpBin::Eq => Some(CompareOp::Eq),
        ast::OpBin::Neq => Some(CompareOp::Neq),
        _ => None,
    }
}

/// Maps general binary operators to bool binary operations.
fn map_op_logic(op: ast::OpBin) -> Option<LogicOp> {
    match op {
        ast::OpBin::And => Some(LogicOp::And),
        ast::OpBin::Or => Some(LogicOp::Or),
        _ => None,
    }
}

/// Checks an expression expecting a Number type.
pub fn check_number(context: &Context, expr: ast::Expr) -> TResult<Expr<NumberT>> {
    match expr {
        ast::Expr::Literal(lit) => match lit {
            ast::Literal::Number(num) => Ok(Expr::Literal(num)),
            _ => Err("Expected number literal".to_string()),
        },
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::Number))?;
            Ok(Expr::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::Number))?;
            Ok(Expr::Call {
                function,
                arguments: typed_args,
            })
        }
        ast::Expr::Binary {
            operator,
            left,
            right,
        } => {
            let op = map_op_arithmetic(operator).ok_or("Invalid operator for number")?;
            let left = check_number(context, *left)?;
            let right = check_number(context, *right)?;
            Ok(Expr::Operator(NumberOps::Arithmetic {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }))
        }
        ast::Expr::Match { scrutinee, arms } => {
            check_match(context, *scrutinee, arms, Type::Number, check_number)
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
        ast::Expr::Unary { .. } => Err("Unexpected expression, expected a number".to_string()),
    }
}

/// Checks an expression expecting a Bool type.
pub fn check_bool(context: &Context, expr: ast::Expr) -> TResult<Expr<BoolT>> {
    match expr {
        ast::Expr::Literal(lit) => match lit {
            ast::Literal::Bool(b) => Ok(Expr::Literal(b)),
            _ => Err("Expected bool literal".to_string()),
        },
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::Bool))?;
            Ok(Expr::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::Bool))?;
            Ok(Expr::Call {
                function,
                arguments: typed_args,
            })
        }
        ast::Expr::Unary {
            operator: ast::OpUn::Not,
            operand,
        } => {
            let operand = check_bool(context, *operand)?;
            Ok(Expr::Operator(BoolOps::Not(Box::new(operand))))
        }
        ast::Expr::Binary {
            operator,
            left,
            right,
        } => {
            if let Some(op) = map_op_logic(operator) {
                let l_bool = check_bool(context, (*left).clone());
                let r_bool = check_bool(context, (*right).clone());
                if let (Ok(l), Ok(r)) = (l_bool, r_bool) {
                    return Ok(Expr::Operator(BoolOps::Logic {
                        op,
                        left: Box::new(l),
                        right: Box::new(r),
                    }));
                }
            }
            let op = map_op_compare(operator).ok_or("Invalid operator for bool")?;
            let left = check_number(context, *left)?;
            let right = check_number(context, *right)?;
            Ok(Expr::Operator(BoolOps::Compare {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }))
        }
        ast::Expr::Match { scrutinee, arms } => {
            check_match(context, *scrutinee, arms, Type::Bool, check_bool)
        }
        ast::Expr::If {
            condition,
            then_branch,
            else_branch,
        } => check_if(context, *condition, *then_branch, *else_branch, check_bool),
        ast::Expr::Field { object, field } => {
            check_field_access(context, object, field, Is(Type::Bool))
        }
    }
}

/// Checks an expression expecting a String type.
pub fn check_string(context: &Context, expr: ast::Expr) -> TResult<Expr<StringT>> {
    match expr {
        ast::Expr::Literal(lit) => match lit {
            ast::Literal::String(s) => Ok(Expr::Literal(s)),
            _ => Err("Expected string literal".to_string()),
        },
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::String))?;
            Ok(Expr::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::String))?;
            Ok(Expr::Call {
                function,
                arguments: typed_args,
            })
        }
        ast::Expr::Match { scrutinee, arms } => {
            check_match(context, *scrutinee, arms, Type::String, check_string)
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
        ast::Expr::Binary { .. } | ast::Expr::Unary { .. } => {
            Err("Unexpected expression, expected a string".to_string())
        }
    }
}

/// Checks an expression expecting a Color type.
pub fn check_color(context: &Context, expr: ast::Expr) -> TResult<Expr<ColorT>> {
    match expr {
        ast::Expr::Literal(lit) => match lit {
            ast::Literal::Color(ast::Color::Rgba { r, g, b, a }) => {
                let r = Box::new(check_number(context, *r)?);
                let g = Box::new(check_number(context, *g)?);
                let b = Box::new(check_number(context, *b)?);
                let a = Box::new(check_number(context, *a)?);
                Ok(Expr::Literal(typed::Color::Rgba { r, g, b, a }))
            }
            _ => Err("Expected color literal".to_string()),
        },
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::Color))?;
            Ok(Expr::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::Color))?;
            Ok(Expr::Call {
                function,
                arguments: typed_args,
            })
        }
        ast::Expr::Match { scrutinee, arms } => {
            check_match(context, *scrutinee, arms, Type::Color, check_color)
        }
        ast::Expr::If {
            condition,
            then_branch,
            else_branch,
        } => check_if(context, *condition, *then_branch, *else_branch, check_color),
        ast::Expr::Field { object, field } => {
            check_field_access(context, object, field, Is(Type::Color))
        }
        ast::Expr::Binary { .. } | ast::Expr::Unary { .. } => {
            Err("Unexpected expression, expected a color".to_string())
        }
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
fn check_field_access<T: SemanticType>(
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
pub fn check_graphic(context: &Context, expr: ast::Expr) -> TResult<Expr<GraphicT>> {
    match expr {
        ast::Expr::Literal(lit) => match lit {
            ast::Literal::Object(ast::Object { name, fields }) => match name.as_str() {
                "Circle" => {
                    let (x, y, radius, color) = extract_fields!(fields, [x, y, radius, color]);
                    Ok(Expr::Literal(typed::Graphic::Circle {
                        x: Box::new(check_number(context, x)?),
                        y: Box::new(check_number(context, y)?),
                        radius: Box::new(check_number(context, radius)?),
                        color: Box::new(check_color(context, color)?),
                    }))
                }
                "Rect" => {
                    let (x, y, width, height, color) =
                        extract_fields!(fields, [x, y, width, height, color]);
                    Ok(Expr::Literal(typed::Graphic::Rect {
                        x: Box::new(check_number(context, x)?),
                        y: Box::new(check_number(context, y)?),
                        width: Box::new(check_number(context, width)?),
                        height: Box::new(check_number(context, height)?),
                        color: Box::new(check_color(context, color)?),
                    }))
                }
                "Text" => {
                    let (x, y, content, color) = extract_fields!(fields, [x, y, content, color]);
                    Ok(Expr::Literal(typed::Graphic::Text {
                        x: Box::new(check_number(context, x)?),
                        y: Box::new(check_number(context, y)?),
                        content: Box::new(check_string(context, content)?),
                        color: Box::new(check_color(context, color)?),
                    }))
                }
                _ => Err("Unexpected object name".to_string()),
            },
            _ => Err("Unexpected expression, expected a graphic".to_string()),
        },
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(Type::Graphic))?;
            Ok(Expr::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(Type::Graphic))?;
            Ok(Expr::Call {
                function,
                arguments: typed_args,
            })
        }
        ast::Expr::Match { scrutinee, arms } => {
            check_match(context, *scrutinee, arms, Type::Graphic, check_graphic)
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
        ast::Expr::Binary { .. } | ast::Expr::Unary { .. } => {
            Err("Unexpected expression, expected a graphic".to_string())
        }
    }
}
