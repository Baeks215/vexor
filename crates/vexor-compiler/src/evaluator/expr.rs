//! Evaluator for expressions

use std::f64::consts::PI;
use std::fmt::Debug;

use crate::evaluator::data_structure::ListNode;
use crate::evaluator::program::eval_assignment;
use crate::evaluator::{Context, EResult, Function, Value, to_int};
use crate::ir::ast::{self, Expr, ListLiteral, Literal, MatchArm, Std, op};
use crate::ir::scene::marker;
use crate::ir::{Number, scene};

pub trait Evaluable {
    type Output: Debug + Clone;
    /// Converts an evaluated output to a [`Value`]
    fn to_value(value: Self::Output) -> Value;
    /// Converts a [`Value`] to an evaluated output
    fn from_value(value: Value) -> EResult<Self::Output>;
    /// Evaluates a literal expression
    fn eval_literal(context: &Context, literal: Literal) -> EResult<Self::Output>;
    /// Matches an evaluated value to a literal ast value
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool>;
    /// Matches an evaluated value to a binary operator pattern
    fn match_bin(
        context: &mut Context,
        scrutinee: Self::Output,
        operator: op::Binary,
        left: Expr,
        right: Expr,
    ) -> EResult<bool>;
}

pub fn eval<T: Evaluable>(context: &Context, expr: ast::Expr) -> EResult<T::Output> {
    match expr {
        Expr::Literal(literal) => T::eval_literal(context, literal),
        Expr::Variable(name) => {
            let value = context.get_var(&name)?;
            T::from_value(value)
        }
        Expr::Const(c) => eval_const::<T>(c),
        Expr::Call { function, args } => eval_call::<T>(context, function, args),
        Expr::Std(std) => eval_std::<T>(context, std),
        Expr::Binary {
            operator,
            left,
            right,
        } => eval_op_bin::<T>(context, operator, *left, *right),
        Expr::Unary { operator, operand } => eval_op_un::<T>(context, operator, *operand),
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

/// Evaluates a binary operator expression
fn eval_op_bin<T: Evaluable>(
    context: &Context,
    operator: op::Binary,
    left: Expr,
    right: Expr,
) -> EResult<T::Output> {
    // Evaluate as general value
    let result = match operator {
        op::Binary::Arithmetic(operator) => {
            // Force evaluate as expected types
            let left = eval::<marker::Number>(context, left)?;
            let right = eval::<marker::Number>(context, right)?;
            Value::Number(match operator {
                op::Arithmetic::Add => left + right,
                op::Arithmetic::Sub => left - right,
                op::Arithmetic::Mul => left * right,
                op::Arithmetic::Div => left / right,
            })
        }
        op::Binary::Logic(operator) => {
            let l = eval::<marker::Bool>(context, left)?;
            let r = eval::<marker::Bool>(context, right)?;
            Value::Bool(match operator {
                op::Logic::And => l && r,
                op::Logic::Or => l || r,
            })
        }
        op::Binary::Compare(operator) => {
            let l = eval::<marker::Number>(context, left)?;
            let r = eval::<marker::Number>(context, right)?;
            Value::Bool(match operator {
                op::Compare::Gt => l > r,
                op::Compare::Gte => l >= r,
                op::Compare::Lt => l < r,
                op::Compare::Lte => l <= r,
                op::Compare::Eq => l == r,
                op::Compare::Neq => l != r,
            })
        }
        op::Binary::Cons => {
            let head = eval::<marker::Any>(context, left)?;
            let tail = eval::<marker::List>(context, right)?;
            Value::List(Box::new(ListNode::Cons(head, tail)))
        }
    };
    // Convert to output type, errors if type mismatch
    T::from_value(result)
}

/// Evaluates a unary operator expression
fn eval_op_un<T: Evaluable>(
    context: &Context,
    operator: op::Unary,
    expr: Expr,
) -> EResult<T::Output> {
    let result = match operator {
        op::Unary::Not => {
            let value = eval::<marker::Bool>(context, expr)?;
            Value::Bool(!value)
        }
    };
    T::from_value(result)
}

/// Evaluates a constant value.
fn eval_const<T: Evaluable>(c: ast::Const) -> Result<<T as Evaluable>::Output, String> {
    T::from_value(match c {
        ast::Const::Pi => Value::Number(PI),
    })
}

/// Evaluates a standard function call.
fn eval_std<T: Evaluable>(context: &Context, std: Std) -> Result<<T as Evaluable>::Output, String> {
    let result = match std {
        Std::Rad(expr) => {
            let x = eval::<marker::Number>(context, *expr)?;
            Value::Number(x.to_radians())
        }
        Std::Sin(expr) => {
            let x = eval::<marker::Number>(context, *expr)?;
            Value::Number(x.sin())
        }
        Std::Cos(expr) => {
            let x = eval::<marker::Number>(context, *expr)?;
            Value::Number(x.cos())
        }
        Std::Tan(expr) => {
            let x = eval::<marker::Number>(context, *expr)?;
            Value::Number(x.tan())
        }
        Std::Map { function, list } => {
            let Expr::Variable(function) = *function else {
                return Err("Must be a function name".to_string());
            };
            let list = eval::<marker::List>(context, *list)?;
            let mut items = vec![];
            let mut curr = *list;
            while let ListNode::Cons(head, tail) = curr {
                let new_head =
                    eval_call_values::<marker::Any>(context, function.clone(), vec![head])?;
                items.push(new_head);
                curr = *tail;
            }

            let mut acc = Box::new(ListNode::Nil);
            for item in items.into_iter().rev() {
                acc = Box::new(ListNode::Cons(item, acc));
            }

            Value::List(acc)
        }
    };
    T::from_value(result)
}

/// Generic function call evaluation.
fn eval_call<T: Evaluable>(context: &Context, func: String, args: Vec<Expr>) -> EResult<T::Output> {
    let args: Vec<Value> = args
        .into_iter()
        .map(|arg_expr| eval::<marker::Any>(context, arg_expr))
        .collect::<Result<Vec<_>, _>>()?;

    eval_call_values::<T>(context, func, args)
}

fn eval_call_values<T: Evaluable>(
    context: &Context,
    func: String,
    args: Vec<Value>,
) -> EResult<T::Output> {
    let Function {
        params,
        scope,
        return_expr,
    } = context.get_function(&func)?;
    // Ensure arguments have correct type
    if params.len() != args.len() {
        return Err("Incorrect number of arguments".to_string());
    }
    let args: Vec<(String, Value)> = params.into_iter().cloned().zip(args).collect();

    // Add arguments to context as variables
    let mut context = context.new_scope_function(args);

    // Evaluate "where" scope of variables
    for assignment in scope {
        eval_assignment(&mut context, assignment.clone())?;
    }

    // Evaluate return expression as the overall expression type
    eval::<T>(&context, return_expr.clone())
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
        Expr::Binary {
            operator,
            left,
            right,
        } => T::match_bin(context, scrutinee, operator, *left, *right),
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
    fn match_bin(
        context: &mut Context,
        scrutinee: Self::Output,
        operator: op::Binary,
        left: Expr,
        right: Expr,
    ) -> EResult<bool> {
        match scrutinee {
            Value::Number(s) => marker::Number::match_bin(context, s, operator, left, right),
            Value::String(s) => marker::String::match_bin(context, s, operator, left, right),
            Value::Bool(s) => marker::Bool::match_bin(context, s, operator, left, right),
            Value::Color(s) => marker::Color::match_bin(context, s, operator, left, right),
            Value::Graphic(s) => marker::Graphic::match_bin(context, s, operator, left, right),
            Value::List(s) => marker::List::match_bin(context, s, operator, left, right),
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
    fn match_bin(
        _: &mut Context,
        _: Self::Output,
        _: op::Binary,
        _: Expr,
        _: Expr,
    ) -> EResult<bool> {
        Err("Pattern not supported".to_string())
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
    fn match_bin(
        _: &mut Context,
        _: Self::Output,
        _: op::Binary,
        _: Expr,
        _: Expr,
    ) -> EResult<bool> {
        Err("Pattern not supported".to_string())
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
    fn match_bin(
        _: &mut Context,
        _: Self::Output,
        _: op::Binary,
        _: Expr,
        _: Expr,
    ) -> EResult<bool> {
        Err("Pattern not supported".to_string())
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
    fn match_bin(
        _: &mut Context,
        _: Self::Output,
        _: op::Binary,
        _: Expr,
        _: Expr,
    ) -> EResult<bool> {
        Err("Pattern not supported".to_string())
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
    fn match_bin(
        _: &mut Context,
        _: Self::Output,
        _: op::Binary,
        _: Expr,
        _: Expr,
    ) -> EResult<bool> {
        Err("Pattern not supported".to_string())
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
            Literal::List(list) => {
                match list {
                    // Build Linked List from vector literal
                    ListLiteral::List(exprs) => {
                        let mut acc = Box::new(ListNode::Nil);

                        // Iterate in reverse to build linked list
                        for e in exprs.into_iter().rev() {
                            let e = eval::<marker::Any>(context, e)?;
                            acc = Box::new(ListNode::Cons(e, acc));
                        }
                        Ok(acc)
                    }
                    // Build Linked List from stepped range
                    ListLiteral::Range { start, second, end } => {
                        // Evaluate range bounds and convert to integers
                        let start = eval::<marker::Number>(context, *start).and_then(to_int)?;
                        let second = second
                            .map(|e| eval::<marker::Number>(context, *e).and_then(to_int))
                            .transpose()?;
                        let end = eval::<marker::Number>(context, *end).and_then(to_int)?;

                        let mut acc = Box::new(ListNode::Nil);

                        // Iterate in reverse to build linked list
                        let iter_rev = build_range_rev(start, second, end)?;
                        for n in iter_rev {
                            // Loss of precision for large numbers
                            let value = Value::Number(n as f64);
                            acc = Box::new(ListNode::Cons(value, acc));
                        }
                        Ok(acc)
                    }
                }
            }
            _ => Err("Expected a list".to_string()),
        }
    }
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::List(ListLiteral::List(ps)) => {
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
    fn match_bin(
        context: &mut Context,
        scrutinee: Self::Output,
        operator: op::Binary,
        left: Expr,
        right: Expr,
    ) -> EResult<bool> {
        match operator {
            op::Binary::Cons => match *scrutinee {
                ListNode::Nil => Ok(false),
                ListNode::Cons(head, tail) => {
                    Ok(match_pattern::<marker::Any>(context, head, left)?
                        && match_pattern::<marker::List>(context, tail, right)?)
                }
            },
            _ => Err("Pattern not supported".to_string()),
        }
    }
}

// --- Helpers --- //

/// Builds a range of integers in reverse
fn build_range_rev(
    start: i64,
    second: Option<i64>,
    end: i64,
) -> EResult<impl Iterator<Item = i64>> {
    let step = match second {
        Some(s) => s - start,
        None => {
            if end >= start {
                1
            } else {
                -1
            }
        }
    };
    let total_range = end - start;

    // Check range step
    if step == 0 {
        return Err("Range step cannot be zero.".to_string());
    }
    if start != end && total_range.signum() != step.signum() {
        return Err("Range step direction is inconsistent with end.".to_string());
    }

    // Normalise end to be the last element in the range
    let end = total_range / step * step + start;

    // Switch to reverse
    let (start, end, step) = (end, start, -step);

    Ok(std::iter::successors(Some(start), move |&prev| {
        let next = prev + step;
        // Check if next value is still within bounds
        if (step > 0 && next <= end) || (step < 0 && next >= end) {
            Some(next)
        } else {
            None
        }
    }))
}
