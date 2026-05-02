//! Evaluator for expressions

use crate::evaluator::program::eval_assignment;
use crate::evaluator::{Context, EResult, Function, Value};
use crate::ir::Number;
use crate::ir::scene;
use crate::ir::typed::expr::{
    ArithmeticOp, BoolOps, CompareOp, Expr, ExprGeneric, LogicOp, MatchArm, NumberOps, Pattern,
    SemanticType,
};
use crate::ir::typed::{self, BoolT, ColorT, GraphicT, NumberT, StringT};

pub fn eval_generic(context: &Context, expr: ExprGeneric) -> EResult<Value> {
    Ok(match expr {
        ExprGeneric::Number(expr) => Value::Number(eval_number(context, expr)?),
        ExprGeneric::String(expr) => Value::String(eval_string(context, expr)?),
        ExprGeneric::Bool(expr) => Value::Bool(eval_bool(context, expr)?),
        ExprGeneric::Color(expr) => Value::Color(eval_color(context, expr)?),
        ExprGeneric::Graphic(expr) => Value::Graphic(eval_graphic(context, expr)?),
    })
}

pub fn eval_number(context: &Context, expr: Expr<NumberT>) -> EResult<Number> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::Number(x) => Ok(x),
            _ => Err("Expected a number".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::Number(x) => Ok(x),
            _ => Err("Expected a number".to_string()),
        },
        Expr::Literal(x) => Ok(x),
        Expr::Operator(NumberOps::Arithmetic { op, left, right }) => match op {
            ArithmeticOp::Add => Ok(eval_number(context, *left)? + eval_number(context, *right)?),
            ArithmeticOp::Sub => Ok(eval_number(context, *left)? - eval_number(context, *right)?),
            ArithmeticOp::Mul => Ok(eval_number(context, *left)? * eval_number(context, *right)?),
            ArithmeticOp::Div => Ok(eval_number(context, *left)? / eval_number(context, *right)?),
        },
        Expr::Match { scrutinee, arms } => {
            let s = eval_number(context, *scrutinee)?;
            eval_match(
                context,
                arms,
                Value::Number(s),
                |ctx, lit| Ok(eval_number(ctx, lit)? == s),
                eval_number,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(context, *condition, *then_branch, *else_branch, eval_number),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_number)
        }
    }
}

/// Generic if-expression evaluation.
fn eval_if<T: SemanticType, U, F>(
    context: &Context,
    condition: Expr<BoolT>,
    then_branch: Expr<T>,
    else_branch: Expr<T>,
    eval_body: F,
) -> EResult<U>
where
    F: Fn(&Context, Expr<T>) -> EResult<U>,
{
    if eval_bool(context, condition)? {
        eval_body(context, then_branch)
    } else {
        eval_body(context, else_branch)
    }
}

/// Generic match-arm evaluation.
///   - `scrutinee_value` is wrapped as a `Value` for binding patterns.
///   - `eq_literal` checks whether a literal pattern matches the scrutinee.
///   - `eval_body` evaluates the arm body.
fn eval_match<E, U, Eq, F>(
    context: &Context,
    arms: Vec<MatchArm<E>>,
    scrutinee_value: Value,
    eq_literal: Eq,
    eval_body: F,
) -> EResult<U>
where
    Eq: Fn(&Context, E) -> EResult<bool>,
    F: Fn(&Context, E) -> EResult<U>,
{
    for MatchArm {
        pattern,
        guard,
        body,
    } in arms
    {
        let scope;
        let arm_ctx: &Context = match pattern {
            Pattern::Binding(name) => {
                scope = context.with_var(name, scrutinee_value.clone());
                &scope
            }
            Pattern::Literal(e) => {
                if !eq_literal(context, e)? {
                    continue;
                }
                context
            }
        };
        if let Some(g) = guard {
            if !eval_bool(arm_ctx, g)? {
                continue;
            }
        }
        return eval_body(arm_ctx, body);
    }
    Err("No match arm matched".to_string())
}

/// Evaluates a field access expression.
fn eval_field_access(context: &Context, object: String, field: String) -> EResult<Value> {
    let object_value = context.get_var(&object)?;
    let result = match object_value {
        Value::Graphic(g) => match g {
            scene::Graphic::Circle {
                x,
                y,
                radius,
                color,
            } => match field.as_str() {
                "x" => Value::Number(x),
                "y" => Value::Number(y),
                "radius" => Value::Number(radius),
                "color" => Value::Color(color),
                _ => return Err("Unknown field".to_string()),
            },
            scene::Graphic::Rect {
                x,
                y,
                width,
                height,
                color,
            } => match field.as_str() {
                "x" => Value::Number(x),
                "y" => Value::Number(y),
                "width" => Value::Number(width),
                "height" => Value::Number(height),
                "color" => Value::Color(color),
                _ => return Err("Unknown field".to_string()),
            },
            scene::Graphic::Text {
                x,
                y,
                content,
                color,
            } => match field.as_str() {
                "x" => Value::Number(x),
                "y" => Value::Number(y),
                "content" => Value::String(content),
                "color" => Value::Color(color),
                _ => return Err("Unknown field".to_string()),
            },
        },
        _ => return Err("Can not access field of this value".to_string()),
    };
    Ok(result)
}

pub fn eval_bool(context: &Context, expr: Expr<BoolT>) -> EResult<bool> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::Bool(x) => Ok(x),
            _ => Err("Expected a bool".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::Bool(x) => Ok(x),
            _ => Err("Expected a bool".to_string()),
        },
        Expr::Literal(b) => Ok(b),
        Expr::Operator(operation) => match operation {
            BoolOps::Compare { op, left, right } => {
                let l = eval_number(context, *left)?;
                let r = eval_number(context, *right)?;
                Ok(match op {
                    CompareOp::Gt => l > r,
                    CompareOp::Gte => l >= r,
                    CompareOp::Lt => l < r,
                    CompareOp::Lte => l <= r,
                    CompareOp::Eq => l == r,
                    CompareOp::Neq => l != r,
                })
            }
            BoolOps::Not(operand) => Ok(!eval_bool(context, *operand)?),
            BoolOps::Logic { op, left, right } => match op {
                LogicOp::And => {
                    // Short-circuit evaluation
                    if !eval_bool(context, *left)? {
                        Ok(false)
                    } else {
                        eval_bool(context, *right)
                    }
                }
                LogicOp::Or => {
                    // Short-circuit evaluation
                    if eval_bool(context, *left)? {
                        Ok(true)
                    } else {
                        eval_bool(context, *right)
                    }
                }
            },
        },
        Expr::Match { scrutinee, arms } => {
            let s = eval_bool(context, *scrutinee)?;
            eval_match(
                context,
                arms,
                Value::Bool(s),
                move |ctx, lit| Ok(eval_bool(ctx, lit)? == s),
                eval_bool,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(context, *condition, *then_branch, *else_branch, eval_bool),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_bool)
        }
    }
}

pub fn eval_string(context: &Context, expr: Expr<StringT>) -> EResult<String> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::String(x) => Ok(x),
            _ => Err("Expected a string".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::String(x) => Ok(x),
            _ => Err("Expected a string".to_string()),
        },
        Expr::Literal(s) => Ok(s),
        Expr::Operator(()) => Err("Operator not supported".to_string()),
        Expr::Match { scrutinee, arms } => {
            let s = eval_string(context, *scrutinee)?;
            let s_cmp = s.clone();
            eval_match(
                context,
                arms,
                Value::String(s),
                move |ctx, lit| Ok(eval_string(ctx, lit)? == s_cmp),
                eval_string,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(context, *condition, *then_branch, *else_branch, eval_string),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_string)
        }
    }
}

pub fn eval_color(context: &Context, expr: Expr<ColorT>) -> EResult<scene::Color> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::Color(x) => Ok(x),
            _ => Err("Expected a color".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::Color(x) => Ok(x),
            _ => Err("Expected a color".to_string()),
        },
        Expr::Literal(typed::Color::Rgba { r, g, b, a }) => Ok(scene::Color::Rgba {
            r: eval_number(context, *r)?,
            g: eval_number(context, *g)?,
            b: eval_number(context, *b)?,
            a: eval_number(context, *a)?,
        }),
        Expr::Operator(()) => Err("Operator not supported".to_string()),
        Expr::Match { scrutinee, arms } => {
            let s = eval_color(context, *scrutinee)?;
            eval_match(
                context,
                arms,
                Value::Color(s),
                move |ctx, lit| Ok(eval_color(ctx, lit)? == s),
                eval_color,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(context, *condition, *then_branch, *else_branch, eval_color),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_color)
        }
    }
}

pub fn eval_graphic(context: &Context, expr: Expr<GraphicT>) -> EResult<scene::Graphic> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::Graphic(x) => Ok(x),
            _ => Err("Expected a graphic".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::Graphic(x) => Ok(x),
            _ => Err("Expected a graphic".to_string()),
        },
        Expr::Literal(l) => match l {
            typed::Graphic::Circle {
                x,
                y,
                radius,
                color,
            } => Ok(scene::Graphic::Circle {
                x: eval_number(context, *x)?,
                y: eval_number(context, *y)?,
                radius: eval_number(context, *radius)?,
                color: eval_color(context, *color)?,
            }),
            typed::Graphic::Rect {
                x,
                y,
                width,
                height,
                color,
            } => Ok(scene::Graphic::Rect {
                x: eval_number(context, *x)?,
                y: eval_number(context, *y)?,
                width: eval_number(context, *width)?,
                height: eval_number(context, *height)?,
                color: eval_color(context, *color)?,
            }),
            typed::Graphic::Text {
                x,
                y,
                content,
                color,
            } => Ok(scene::Graphic::Text {
                x: eval_number(context, *x)?,
                y: eval_number(context, *y)?,
                content: eval_string(context, *content)?,
                color: eval_color(context, *color)?,
            }),
        },
        Expr::Operator(()) => Err("Operator not supported".to_string()),
        Expr::Match { scrutinee, arms } => {
            let s = eval_graphic(context, *scrutinee)?;
            let s_cmp = s.clone();
            eval_match(
                context,
                arms,
                Value::Graphic(s),
                move |ctx, lit| Ok(eval_graphic(ctx, lit)? == s_cmp),
                eval_graphic,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(
            context,
            *condition,
            *then_branch,
            *else_branch,
            eval_graphic,
        ),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_graphic)
        }
    }
}

fn eval_call(context: &Context, func: String, args: Vec<ExprGeneric>) -> EResult<Value> {
    let Function {
        params,
        scope,
        return_expr,
    } = context.get_function(&func)?;
    let mut context = context.new_scope_function(
        &params,
        args.into_iter()
            .map(|a| eval_generic(context, a))
            .collect::<Result<Vec<Value>, _>>()?,
    );

    for assignment in scope {
        eval_assignment(&mut context, assignment.clone())?;
    }
    eval_generic(&context, return_expr.clone())
}
