//! Evaluator for expressions

use std::fmt::Debug;

use crate::evaluator::program::eval_assignment;
use crate::evaluator::{Context, EResult, Function, Value};
use crate::ir::ast::{self, Expr, Literal, MatchArm, OpBin, OpUn};
use crate::ir::scene::marker;
use crate::ir::{ListNode, Number, scene};

pub trait Evaluable {
    type Output: Debug + Clone;
    /// Converts an evaluated output to a [`Value`]
    fn to_value(value: Self::Output) -> Value;
    /// Converts a [`Value`] to an evaluated output
    fn from_value(value: Value) -> EResult<Self::Output>;
    /// Evaluates a literal expression
    fn eval_literal(context: &Context, literal: Literal) -> EResult<Self::Output>;
    /// Evaluates a binary operator expression
    fn eval_op_bin(
        context: &Context,
        operator: OpBin,
        left: Expr,
        right: Expr,
    ) -> EResult<Self::Output>;
    /// Evaluates a unary operator expression
    fn eval_op_un(context: &Context, operator: OpUn, expr: Expr) -> EResult<Self::Output>;
    /// Matches an evaluated value to a literal ast value
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool>;
}

pub fn eval<T: Evaluable>(context: &Context, expr: ast::Expr) -> EResult<T::Output> {
    match expr {
        Expr::Literal(literal) => T::eval_literal(context, literal),
        Expr::Variable(name) => {
            let value = context.get_var(&name)?;
            T::from_value(value)
        }
        Expr::Call { function, args } => eval_call::<T>(context, function, args),
        Expr::Binary {
            operator,
            left,
            right,
        } => T::eval_op_bin(context, operator, *left, *right),
        Expr::Unary { operator, operand } => T::eval_op_un(context, operator, *operand),
        Expr::Match { scrutinee, arms } => {
            let s = eval::<marker::Any>(context, *scrutinee)?;
            eval_match::<T>(context, arms, s)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if::<T>(context, *condition, *then_branch, *else_branch),
        Expr::Field { object, field } => eval_field_access::<T>(context, object, field),
    }
}

/// Generic function call evaluation.
fn eval_call<T: Evaluable>(context: &Context, func: String, args: Vec<Expr>) -> EResult<T::Output> {
    let Function {
        params,
        scope,
        return_expr,
    } = context.get_function(&func)?;
    // Ensure arguments have correct type
    if params.len() != args.len() {
        return Err("Incorrect number of arguments".to_string());
    }
    let args: Vec<(String, Value)> = params
        .iter()
        .zip(args)
        .map(|(name, arg_expr)| {
            eval::<marker::Any>(context, arg_expr).map(|arg| (name.clone(), arg))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Add arguments to context as variables
    let mut context = context.new_scope_function(args);

    // Evaluate "where" scope of variables
    for assignment in scope {
        eval_assignment(&mut context, assignment.clone())?;
    }
    // Evaluate return expression
    let value = eval::<marker::Any>(&context, return_expr.clone())?;
    T::from_value(value)
}

fn match_pattern<T: Evaluable>(
    context: &mut Context,
    scrutinee: T::Output,
    pattern: Expr,
) -> EResult<bool> {
    match pattern {
        Expr::Variable(name) => {
            context.set_var(name, T::to_value(scrutinee));
            Ok(true)
        }
        Expr::Literal(lit_pattern) => T::match_literal(context, scrutinee, lit_pattern),
        _ => Err("Pattern not supported".to_string()),
    }
}

/// Generic match-arm evaluation.
fn eval_match<T: Evaluable>(
    context: &Context,
    arms: Vec<MatchArm>,
    scrutinee: Value,
) -> EResult<T::Output> {
    for MatchArm {
        pattern,
        guard,
        body,
    } in arms
    {
        let mut arm_ctx = context.clone();
        let matched = match_pattern::<marker::Any>(&mut arm_ctx, scrutinee.clone(), pattern)?;
        if !matched {
            continue;
        }

        if let Some(condition) = guard {
            if !eval::<marker::Bool>(&arm_ctx, condition)? {
                continue;
            }
        }
        return eval::<T>(&arm_ctx, body);
    }
    Err("No match arm matched".to_string())
}

/// Generic if-expression evaluation.
fn eval_if<T: Evaluable>(
    context: &Context,
    condition: Expr,
    then_branch: Expr,
    else_branch: Expr,
) -> EResult<T::Output> {
    if eval::<marker::Bool>(context, condition)? {
        eval::<T>(context, then_branch)
    } else {
        eval::<T>(context, else_branch)
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

impl Evaluable for marker::Any {
    type Output = Value;
    fn to_value(value: Self::Output) -> Value {
        value
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        Ok(value)
    }
    fn eval_literal(context: &Context, literal: Literal) -> EResult<Self::Output> {
        Ok(match literal {
            Literal::Number(n) => Value::Number(n),
            Literal::String(s) => Value::String(s),
            Literal::Bool(b) => Value::Bool(b),
            Literal::Color(_) => Value::Color(marker::Color::eval_literal(context, literal)?),
            Literal::Graphic(_) => Value::Graphic(marker::Graphic::eval_literal(context, literal)?),
            Literal::List(_) => Value::List(marker::List::eval_literal(context, literal)?),
        })
    }
    fn eval_op_bin(
        context: &Context,
        operator: OpBin,
        left: Expr,
        right: Expr,
    ) -> EResult<Self::Output> {
        match operator {
            OpBin::Add | OpBin::Sub | OpBin::Mul | OpBin::Div => {
                marker::Number::eval_op_bin(context, operator, left, right).map(Value::Number)
            }
            OpBin::Gt
            | OpBin::Gte
            | OpBin::Lt
            | OpBin::Lte
            | OpBin::Eq
            | OpBin::Neq
            | OpBin::And
            | OpBin::Or => {
                marker::Bool::eval_op_bin(context, operator, left, right).map(Value::Bool)
            }
            OpBin::Cons => {
                marker::List::eval_op_bin(context, operator, left, right).map(Value::List)
            }
        }
    }
    fn eval_op_un(context: &Context, operator: OpUn, expr: Expr) -> EResult<Self::Output> {
        match operator {
            OpUn::Not => marker::Bool::eval_op_un(context, operator, expr).map(Value::Bool),
        }
    }
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match scrutinee {
            Value::Number(s) => marker::Number::match_literal(context, s, literal_pattern),
            Value::String(s) => marker::String::match_literal(context, s, literal_pattern),
            Value::Bool(s) => marker::Bool::match_literal(context, s, literal_pattern),
            Value::Color(s) => marker::Color::match_literal(context, s, literal_pattern),
            Value::Graphic(s) => marker::Graphic::match_literal(context, s, literal_pattern),
            Value::List(s) => marker::List::match_literal(context, s, literal_pattern),
        }
    }
}

impl Evaluable for marker::Number {
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
    fn eval_literal(_: &Context, literal: Literal) -> EResult<Self::Output> {
        match literal {
            Literal::Number(n) => Ok(n),
            _ => Err("Expected a number".to_string()),
        }
    }
    fn eval_op_bin(
        context: &Context,
        operator: OpBin,
        left: Expr,
        right: Expr,
    ) -> EResult<Self::Output> {
        let left = eval::<marker::Number>(context, left)?;
        let right = eval::<marker::Number>(context, right)?;
        match operator {
            OpBin::Add => Ok(left + right),
            OpBin::Sub => Ok(left - right),
            OpBin::Mul => Ok(left * right),
            OpBin::Div => Ok(left / right),
            _ => Err("Unsupported operator for number".to_string()),
        }
    }
    fn eval_op_un(_: &Context, _: OpUn, _: Expr) -> EResult<Self::Output> {
        Err("Unsupported operator".to_string())
    }
    fn match_literal(
        _: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::Number(n) => Ok(scrutinee == n),
            _ => Err("Expected a number literal".to_string()),
        }
    }
}

impl Evaluable for marker::String {
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
    fn eval_literal(_: &Context, literal: Literal) -> EResult<Self::Output> {
        match literal {
            Literal::String(s) => Ok(s),
            _ => Err("Expected a string".to_string()),
        }
    }
    fn eval_op_bin(_: &Context, _: OpBin, _: Expr, _: Expr) -> EResult<Self::Output> {
        Err("Unsupported operator".to_string())
    }
    fn eval_op_un(_: &Context, _: OpUn, _: Expr) -> EResult<Self::Output> {
        Err("Unsupported operator".to_string())
    }
    fn match_literal(
        _: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::String(s) => Ok(scrutinee == s),
            _ => Err("Expected a string literal".to_string()),
        }
    }
}

impl Evaluable for marker::Bool {
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
    fn eval_literal(_: &Context, literal: Literal) -> EResult<Self::Output> {
        match literal {
            Literal::Bool(b) => Ok(b),
            _ => Err("Expected a bool".to_string()),
        }
    }
    fn eval_op_bin(
        context: &Context,
        operator: OpBin,
        left: Expr,
        right: Expr,
    ) -> EResult<Self::Output> {
        match operator {
            OpBin::And | OpBin::Or => {
                let l = eval::<marker::Bool>(context, left)?;
                let r = eval::<marker::Bool>(context, right)?;
                Ok(match operator {
                    OpBin::And => l && r,
                    OpBin::Or => l || r,
                    _ => unreachable!(),
                })
            }
            OpBin::Gt | OpBin::Gte | OpBin::Lt | OpBin::Lte | OpBin::Eq | OpBin::Neq => {
                let l = eval::<marker::Number>(context, left)?;
                let r = eval::<marker::Number>(context, right)?;
                Ok(match operator {
                    OpBin::Gt => l > r,
                    OpBin::Gte => l >= r,
                    OpBin::Lt => l < r,
                    OpBin::Lte => l <= r,
                    OpBin::Eq => l == r,
                    OpBin::Neq => l != r,
                    _ => unreachable!(),
                })
            }
            _ => Err("Unsupported operator".to_string()),
        }
    }
    fn eval_op_un(context: &Context, operator: OpUn, expr: Expr) -> EResult<Self::Output> {
        match operator {
            OpUn::Not => {
                let value = eval::<marker::Bool>(context, expr)?;
                Ok(!value)
            }
        }
    }
    fn match_literal(
        _: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::Bool(b) => Ok(scrutinee == b),
            _ => Err("Expected a bool literal".to_string()),
        }
    }
}

impl Evaluable for marker::Color {
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
    fn eval_literal(context: &Context, literal: Literal) -> EResult<Self::Output> {
        let Literal::Color(ast::Color::Rgba { r, g, b, a }) = literal else {
            return Err("Expected a color".to_string());
        };
        Ok(scene::Color::Rgba {
            r: eval::<marker::Number>(context, *r)?,
            g: eval::<marker::Number>(context, *g)?,
            b: eval::<marker::Number>(context, *b)?,
            a: eval::<marker::Number>(context, *a)?,
        })
    }
    fn eval_op_bin(_: &Context, _: OpBin, _: Expr, _: Expr) -> EResult<Self::Output> {
        Err("Unsupported operator".to_string())
    }
    fn eval_op_un(_: &Context, _: OpUn, _: Expr) -> EResult<Self::Output> {
        Err("Unsupported operator".to_string())
    }
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::Color(c) => {
                let scene::Color::Rgba { r, g, b, a } = scrutinee;
                let ast::Color::Rgba {
                    r: r_expr,
                    g: g_expr,
                    b: b_expr,
                    a: a_expr,
                } = c;
                Ok(match_pattern::<marker::Number>(context, r, *r_expr)?
                    && match_pattern::<marker::Number>(context, g, *g_expr)?
                    && match_pattern::<marker::Number>(context, b, *b_expr)?
                    && match_pattern::<marker::Number>(context, a, *a_expr)?)
            }
            _ => Err("Expected a color literal".to_string()),
        }
    }
}

impl Evaluable for marker::Graphic {
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
    fn eval_literal(context: &Context, literal: Literal) -> EResult<Self::Output> {
        let Literal::Graphic(node) = literal else {
            return Err("Expected a graphic object".to_string());
        };
        match node {
            ast::Graphic::Circle {
                x,
                y,
                radius,
                color,
            } => Ok(scene::Graphic::Circle {
                x: eval::<marker::Number>(context, *x)?,
                y: eval::<marker::Number>(context, *y)?,
                radius: eval::<marker::Number>(context, *radius)?,
                color: eval::<marker::Color>(context, *color)?,
            }),
            ast::Graphic::Rect {
                x,
                y,
                width,
                height,
                color,
            } => Ok(scene::Graphic::Rect {
                x: eval::<marker::Number>(context, *x)?,
                y: eval::<marker::Number>(context, *y)?,
                width: eval::<marker::Number>(context, *width)?,
                height: eval::<marker::Number>(context, *height)?,
                color: eval::<marker::Color>(context, *color)?,
            }),
            ast::Graphic::Text {
                x,
                y,
                content,
                color,
            } => Ok(scene::Graphic::Text {
                x: eval::<marker::Number>(context, *x)?,
                y: eval::<marker::Number>(context, *y)?,
                content: eval::<marker::String>(context, *content)?,
                color: eval::<marker::Color>(context, *color)?,
            }),
        }
    }
    fn eval_op_bin(_: &Context, _: OpBin, _: Expr, _: Expr) -> EResult<Self::Output> {
        Err("Unsupported operator".to_string())
    }
    fn eval_op_un(_: &Context, _: OpUn, _: Expr) -> EResult<Self::Output> {
        Err("Unsupported operator".to_string())
    }
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::Graphic(pattern) => match (scrutinee, pattern) {
                (
                    scene::Graphic::Circle {
                        x,
                        y,
                        radius,
                        color,
                    },
                    ast::Graphic::Circle {
                        x: x_e,
                        y: y_e,
                        radius: radius_e,
                        color: color_e,
                    },
                ) => Ok(match_pattern::<marker::Number>(context, x, *x_e)?
                    && match_pattern::<marker::Number>(context, y, *y_e)?
                    && match_pattern::<marker::Number>(context, radius, *radius_e)?
                    && match_pattern::<marker::Color>(context, color, *color_e)?),
                (
                    scene::Graphic::Rect {
                        x,
                        y,
                        width,
                        height,
                        color,
                    },
                    ast::Graphic::Rect {
                        x: x_e,
                        y: y_e,
                        width: width_e,
                        height: height_e,
                        color: color_e,
                    },
                ) => Ok(match_pattern::<marker::Number>(context, x, *x_e)?
                    && match_pattern::<marker::Number>(context, y, *y_e)?
                    && match_pattern::<marker::Number>(context, width, *width_e)?
                    && match_pattern::<marker::Number>(context, height, *height_e)?
                    && match_pattern::<marker::Color>(context, color, *color_e)?),
                (
                    scene::Graphic::Text {
                        x,
                        y,
                        content,
                        color,
                    },
                    ast::Graphic::Text {
                        x: x_e,
                        y: y_e,
                        content: content_e,
                        color: color_e,
                    },
                ) => Ok(match_pattern::<marker::Number>(context, x, *x_e)?
                    && match_pattern::<marker::Number>(context, y, *y_e)?
                    && match_pattern::<marker::String>(context, content, *content_e)?
                    && match_pattern::<marker::Color>(context, color, *color_e)?),
                _ => Ok(false),
            },
            _ => Err("Expected a graphic literal".to_string()),
        }
    }
}

impl Evaluable for marker::List {
    type Output = Box<ListNode<Value>>;
    fn to_value(value: Self::Output) -> Value {
        Value::List(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::List(l) => Ok(l),
            _ => Err("Expected a list".to_string()),
        }
    }
    fn eval_literal(context: &Context, literal: Literal) -> EResult<Self::Output> {
        match literal {
            Literal::List(exprs) => {
                // Build Linked List from vector literal
                let mut acc = Box::new(ListNode::Nil);
                for e in exprs.into_iter().rev() {
                    let e = eval::<marker::Any>(context, e)?;
                    acc = Box::new(ListNode::Cons(e, acc));
                }
                Ok(acc)
            }
            _ => Err("Expected a list".to_string()),
        }
    }
    fn eval_op_bin(
        context: &Context,
        operator: OpBin,
        left: Expr,
        right: Expr,
    ) -> EResult<Self::Output> {
        match operator {
            OpBin::Cons => {
                let head = eval::<marker::Any>(context, left)?;
                let tail = eval::<marker::List>(context, right)?;
                Ok(Box::new(ListNode::Cons(head, tail)))
            }
            _ => Err("Unsupported operator".to_string()),
        }
    }
    fn eval_op_un(_: &Context, _: OpUn, _: Expr) -> EResult<Self::Output> {
        Err("Unsupported operator".to_string())
    }
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::List(ps) => {
                let mut node = scrutinee;
                for item_pattern in ps.into_iter() {
                    let ListNode::Cons(head, tail) = *node else {
                        // Scrutinee is Nil, pattern is too long
                        return Ok(false);
                    };
                    let matched = match_pattern::<marker::Any>(context, head, item_pattern)?;
                    if !matched {
                        return Ok(false);
                    }
                    node = tail;
                }
                match *node {
                    ListNode::Nil => Ok(true),
                    // Scrutinee still has items left, pattern is too short
                    ListNode::Cons(_, _) => Ok(false),
                }
            }
            _ => Err("Expected a list literal".to_string()),
        }
    }
}
