//! Evaluator for expressions

use std::fmt::Debug;

use crate::evaluator::program::eval_assignment;
use crate::evaluator::{Context, EResult, Function, Value};
use crate::ir::Number;
use crate::ir::scene;
use crate::ir::typed::expr::{
    ArithmeticOp, BoolOps, CompareOp, Expr, ExprGeneric, LogicOp, MatchArm, NumberOps, Pattern,
    SemanticType,
};
use crate::ir::typed::{self, BoolT, ColorT, GraphicT, NumberT, StringT};

pub trait Evaluable: SemanticType {
    type Output: Debug + Clone;
    /// Converts an evaluated output to a [`Value`]
    fn to_value(value: Self::Output) -> Value;
    /// Converts a [`Value`] to an evaluated output
    fn from_value(value: Value) -> EResult<Self::Output>;
    /// Evaluates a literal expression
    fn eval_literal(context: &Context, literal: Self::NativeType) -> EResult<Self::Output>;
    /// Evaluates an operator node
    fn eval_operator(context: &Context, op_node: Self::OperatorNode) -> EResult<Self::Output>;
    /// Matches an evaluated value to a literal typed node
    ///   Return error if not matched
    fn match_literal(scrutinee: Self::Output, expected: Self::NativeType) -> EResult<()>;
}

pub fn eval_generic(context: &Context, expr: ExprGeneric) -> EResult<Value> {
    Ok(match expr {
        ExprGeneric::Number(expr) => Value::Number(eval(context, expr)?),
        ExprGeneric::String(expr) => Value::String(eval(context, expr)?),
        ExprGeneric::Bool(expr) => Value::Bool(eval(context, expr)?),
        ExprGeneric::Color(expr) => Value::Color(eval(context, expr)?),
        ExprGeneric::Graphic(expr) => Value::Graphic(eval(context, expr)?),
    })
}

pub fn eval<T: Evaluable>(context: &Context, expr: Expr<T>) -> EResult<T::Output> {
    match expr {
        Expr::Literal(literal) => T::eval_literal(context, literal),
        Expr::Variable(name) => {
            let value = context.get_var(&name)?;
            T::from_value(value)
        }
        Expr::Call {
            function,
            arguments,
        } => eval_call::<T>(context, function, arguments),
        Expr::Operator(op) => T::eval_operator(context, op),
        Expr::Match { scrutinee, arms } => {
            let s = eval(context, *scrutinee)?;
            eval_match(context, arms, s)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(context, *condition, *then_branch, *else_branch),
        Expr::Field { object, field } => eval_field_access::<T>(context, object, field),
    }
}

/// Generic function call evaluation.
fn eval_call<T: Evaluable>(
    context: &Context,
    func: String,
    args: Vec<ExprGeneric>,
) -> EResult<T::Output> {
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
    let value = eval_generic(&context, return_expr.clone())?;
    T::from_value(value)
}

/// Generic match-arm evaluation.
fn eval_match<T: Evaluable>(
    context: &Context,
    arms: Vec<MatchArm<T>>,
    scrutinee: T::Output,
) -> EResult<T::Output> {
    for MatchArm {
        pattern,
        guard,
        body,
    } in arms
    {
        let scope;
        let arm_ctx: &Context = match pattern {
            Pattern::Binding(name) => {
                scope = context.with_var(name, T::to_value(scrutinee.clone()));
                &scope
            }
            Pattern::Literal(expected) => {
                if let Err(_) = T::match_literal(scrutinee.clone(), expected) {
                    continue;
                }
                context
            }
        };
        if let Some(condition) = guard {
            if !eval::<BoolT>(arm_ctx, condition)? {
                continue;
            }
        }
        return eval(arm_ctx, body);
    }
    Err("No match arm matched".to_string())
}

/// Generic if-expression evaluation.
fn eval_if<T: Evaluable>(
    context: &Context,
    condition: Expr<BoolT>,
    then_branch: Expr<T>,
    else_branch: Expr<T>,
) -> EResult<T::Output> {
    if eval::<BoolT>(context, condition)? {
        eval(context, then_branch)
    } else {
        eval(context, else_branch)
    }
}

/// Evaluates a field access expression.
fn eval_field_access<T: Evaluable>(
    context: &Context,
    object: String,
    field: String,
) -> EResult<T::Output> {
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
    T::from_value(result)
}

/// Extracts literal from an expr and matches to an evaluated value.
fn match_literal_expr<T: Evaluable>(scrutinee: T::Output, expected: Expr<T>) -> EResult<()> {
    match expected {
        Expr::Literal(literal) => T::match_literal(scrutinee, literal),
        _ => Err("Expected a literal".to_string()),
    }
}

impl Evaluable for NumberT {
    type Output = Number;
    fn to_value(value: Self::Output) -> Value {
        Value::Number(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Number(x) => Ok(x),
            _ => Err("Expected a number".to_string()),
        }
    }
    fn eval_literal(_: &Context, literal: Self::NativeType) -> EResult<Self::Output> {
        Ok(literal)
    }
    fn eval_operator(context: &Context, op_node: Self::OperatorNode) -> EResult<Self::Output> {
        match op_node {
            NumberOps::Arithmetic { op, left, right } => {
                let left = eval(context, *left)?;
                let right = eval(context, *right)?;
                Ok(match op {
                    ArithmeticOp::Add => left + right,
                    ArithmeticOp::Sub => left - right,
                    ArithmeticOp::Mul => left * right,
                    ArithmeticOp::Div => left / right,
                })
            }
        }
    }
    fn match_literal(scrutinee: Self::Output, expected: Self::NativeType) -> EResult<()> {
        if scrutinee == expected {
            Ok(())
        } else {
            Err("Literal mismatch".to_string())
        }
    }
}

impl Evaluable for StringT {
    type Output = String;
    fn to_value(value: Self::Output) -> Value {
        Value::String(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err("Expected a string".to_string()),
        }
    }
    fn eval_literal(_: &Context, literal: Self::NativeType) -> EResult<Self::Output> {
        Ok(literal)
    }
    fn eval_operator(_: &Context, op_node: Self::OperatorNode) -> EResult<Self::Output> {
        match op_node {
            () => Err("Operator not supported".to_string()),
        }
    }
    fn match_literal(scrutinee: Self::Output, expected: Self::NativeType) -> EResult<()> {
        if scrutinee == expected {
            Ok(())
        } else {
            Err("Literal mismatch".to_string())
        }
    }
}

impl Evaluable for BoolT {
    type Output = bool;
    fn to_value(value: Self::Output) -> Value {
        Value::Bool(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Bool(x) => Ok(x),
            _ => Err("Expected a bool".to_string()),
        }
    }
    fn eval_literal(_: &Context, literal: Self::NativeType) -> EResult<Self::Output> {
        Ok(literal)
    }
    fn eval_operator(context: &Context, op_node: Self::OperatorNode) -> EResult<Self::Output> {
        match op_node {
            BoolOps::Compare { op, left, right } => {
                let l = eval(context, *left)?;
                let r = eval(context, *right)?;
                Ok(match op {
                    CompareOp::Gt => l > r,
                    CompareOp::Gte => l >= r,
                    CompareOp::Lt => l < r,
                    CompareOp::Lte => l <= r,
                    CompareOp::Eq => l == r,
                    CompareOp::Neq => l != r,
                })
            }
            BoolOps::Not(operand) => Ok(!eval(context, *operand)?),
            BoolOps::Logic { op, left, right } => match op {
                LogicOp::And => {
                    // Short-circuit evaluation
                    if !eval(context, *left)? {
                        Ok(false)
                    } else {
                        eval(context, *right)
                    }
                }
                LogicOp::Or => {
                    // Short-circuit evaluation
                    if eval(context, *left)? {
                        Ok(true)
                    } else {
                        eval(context, *right)
                    }
                }
            },
        }
    }
    fn match_literal(scrutinee: Self::Output, expected: Self::NativeType) -> EResult<()> {
        if scrutinee == expected {
            Ok(())
        } else {
            Err("Literal mismatch".to_string())
        }
    }
}

impl Evaluable for ColorT {
    type Output = scene::Color;
    fn to_value(value: Self::Output) -> Value {
        Value::Color(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Color(x) => Ok(x),
            _ => Err("Expected a color".to_string()),
        }
    }
    fn eval_literal(context: &Context, literal: Self::NativeType) -> EResult<Self::Output> {
        let typed::Color::Rgba { r, g, b, a } = literal;
        Ok(scene::Color::Rgba {
            r: eval(context, *r)?,
            g: eval(context, *g)?,
            b: eval(context, *b)?,
            a: eval(context, *a)?,
        })
    }
    fn eval_operator(_: &Context, op_node: Self::OperatorNode) -> EResult<Self::Output> {
        match op_node {
            () => Err("Operator not supported".to_string()),
        }
    }
    fn match_literal(scrutinee: Self::Output, expected: Self::NativeType) -> EResult<()> {
        let scene::Color::Rgba { r, g, b, a } = scrutinee;
        let typed::Color::Rgba {
            r: r_e,
            g: g_e,
            b: b_e,
            a: a_e,
        } = expected;
        // All fields must be literals
        match_literal_expr(r, *r_e)?;
        match_literal_expr(g, *g_e)?;
        match_literal_expr(b, *b_e)?;
        match_literal_expr(a, *a_e)
    }
}

impl Evaluable for GraphicT {
    type Output = scene::Graphic;
    fn to_value(value: Self::Output) -> Value {
        Value::Graphic(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Graphic(x) => Ok(x),
            _ => Err("Expected a graphic".to_string()),
        }
    }
    fn eval_literal(context: &Context, literal: Self::NativeType) -> EResult<Self::Output> {
        match literal {
            typed::Graphic::Circle {
                x,
                y,
                radius,
                color,
            } => Ok(scene::Graphic::Circle {
                x: eval(context, *x)?,
                y: eval(context, *y)?,
                radius: eval(context, *radius)?,
                color: eval(context, *color)?,
            }),
            typed::Graphic::Rect {
                x,
                y,
                width,
                height,
                color,
            } => Ok(scene::Graphic::Rect {
                x: eval(context, *x)?,
                y: eval(context, *y)?,
                width: eval(context, *width)?,
                height: eval(context, *height)?,
                color: eval(context, *color)?,
            }),
            typed::Graphic::Text {
                x,
                y,
                content,
                color,
            } => Ok(scene::Graphic::Text {
                x: eval(context, *x)?,
                y: eval(context, *y)?,
                content: eval(context, *content)?,
                color: eval(context, *color)?,
            }),
        }
    }
    fn eval_operator(_: &Context, op_node: Self::OperatorNode) -> EResult<Self::Output> {
        match op_node {
            () => Err("Operator not supported".to_string()),
        }
    }
    fn match_literal(scrutinee: Self::Output, expected: Self::NativeType) -> EResult<()> {
        match (scrutinee, expected) {
            (
                scene::Graphic::Circle {
                    x,
                    y,
                    radius,
                    color,
                },
                typed::Graphic::Circle {
                    x: x_e,
                    y: y_e,
                    radius: radius_e,
                    color: color_e,
                },
            ) => {
                match_literal_expr(x, *x_e)?;
                match_literal_expr(y, *y_e)?;
                match_literal_expr(radius, *radius_e)?;
                match_literal_expr(color, *color_e)
            }
            (
                scene::Graphic::Rect {
                    x,
                    y,
                    width,
                    height,
                    color,
                },
                typed::Graphic::Rect {
                    x: x_e,
                    y: y_e,
                    width: width_e,
                    height: height_e,
                    color: color_e,
                },
            ) => {
                match_literal_expr(x, *x_e)?;
                match_literal_expr(y, *y_e)?;
                match_literal_expr(width, *width_e)?;
                match_literal_expr(height, *height_e)?;
                match_literal_expr(color, *color_e)
            }
            (
                scene::Graphic::Text {
                    x,
                    y,
                    content,
                    color,
                },
                typed::Graphic::Text {
                    x: x_e,
                    y: y_e,
                    content: content_e,
                    color: color_e,
                },
            ) => {
                match_literal_expr(x, *x_e)?;
                match_literal_expr(y, *y_e)?;
                match_literal_expr(content, *content_e)?;
                match_literal_expr(color, *color_e)
            }
            _ => Err("Not a literal pattern".to_string()),
        }
    }
}
