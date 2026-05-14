//! Evaluator for expressions

use std::f64::consts::PI;
use std::fmt::Debug;

use crate::evaluator::{EResult, EnvExt, EnvRef, Value, ty};
use crate::ir::ast::{self, Expr, Literal, MatchArm, op};
use crate::ir::scene;

mod bool;
mod color;
mod function;
mod graphic;
mod list;
mod number;
mod string;

pub use function::Callable;
use function::eval_call;

pub trait Evaluable {
    type Output: Debug + Clone;
    /// Converts an evaluated output to a [`Value`]
    fn to_value(value: Self::Output) -> Value;
    /// Converts a [`Value`] to an evaluated output
    fn from_value(value: Value) -> EResult<Self::Output>;
    /// Evaluates a literal expression
    fn eval_literal(env: &EnvRef, literal: Literal) -> EResult<Self::Output>;
    /// Matches an evaluated value to a literal ast value
    fn match_literal(
        env: &EnvRef,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool>;
    /// Matches an evaluated value to a binary operator pattern
    fn match_bin(
        env: &EnvRef,
        scrutinee: Self::Output,
        operator: op::Binary,
        left: Expr,
        right: Expr,
    ) -> EResult<bool>;
    /// Matches an evaluated value to a function call
    fn match_call(
        env: &EnvRef,
        scrutinee: Self::Output,
        function: Expr,
        args: Vec<Expr>,
    ) -> EResult<bool>;
}

pub fn eval<T: Evaluable>(env: &EnvRef, expr: ast::Expr) -> EResult<T::Output> {
    match expr {
        Expr::Literal(literal) => T::eval_literal(env, literal),
        Expr::Variable(name) => {
            let value = env.get_var(&name)?;
            T::from_value(value)
        }
        Expr::Const(c) => eval_const::<T>(c),
        Expr::Call { function, args } => {
            let function = eval::<ty::Function>(env, *function)?;
            let args: Vec<Value> = args
                .into_iter()
                .map(|arg_expr| eval::<ty::Any>(env, arg_expr))
                .collect::<Result<Vec<_>, _>>()?;

            eval_call::<T>(env, function, args)
        }
        Expr::Function(func) => {
            let func_value = Value::Function(Callable::User {
                func,
                // Capture the current environment
                closure_env: env.clone(), // Cloned reference
            });
            T::from_value(func_value)
        }
        Expr::Std(func) => T::from_value(Value::Function(Callable::Std(func))),
        Expr::Binary {
            operator,
            left,
            right,
        } => eval_op_bin::<T>(env, operator, *left, *right),
        Expr::Unary { operator, operand } => eval_op_un::<T>(env, operator, *operand),
        Expr::Match { scrutinee, arms } => {
            let s = eval::<ty::Any>(env, *scrutinee)?;
            eval_match::<T>(env, arms, s)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            if eval::<ty::Bool>(env, *condition)? {
                eval::<T>(env, *then_branch)
            } else {
                eval::<T>(env, *else_branch)
            }
        }
        Expr::Field { object, field } => eval_field_access::<T>(env, object, field),
    }
}

/// Evaluates a binary operator expression
fn eval_op_bin<T: Evaluable>(
    env: &EnvRef,
    operator: op::Binary,
    left: Expr,
    right: Expr,
) -> EResult<T::Output> {
    // Evaluate as general value
    let result = match operator {
        op::Binary::Arithmetic(operator) => {
            // Force evaluate as expected types
            let left = eval::<ty::Number>(env, left)?;
            let right = eval::<ty::Number>(env, right)?;
            Value::Number(match operator {
                op::Arithmetic::Add => left + right,
                op::Arithmetic::Sub => left - right,
                op::Arithmetic::Mul => left * right,
                op::Arithmetic::Div => left / right,
            })
        }
        op::Binary::Logic(operator) => {
            let l = eval::<ty::Bool>(env, left)?;
            let r = eval::<ty::Bool>(env, right)?;
            Value::Bool(match operator {
                op::Logic::And => l && r,
                op::Logic::Or => l || r,
            })
        }
        op::Binary::Compare(operator) => {
            let l = eval::<ty::Number>(env, left)?;
            let r = eval::<ty::Number>(env, right)?;
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
            let head = eval::<ty::Any>(env, left)?;
            let tail = eval::<ty::List>(env, right)?;
            Value::List(Box::new(list::ListNode::Cons(head, tail)))
        }
    };
    // Convert to output type, errors if type mismatch
    T::from_value(result)
}

/// Evaluates a unary operator expression
fn eval_op_un<T: Evaluable>(env: &EnvRef, operator: op::Unary, expr: Expr) -> EResult<T::Output> {
    let result = match operator {
        op::Unary::Not => {
            let value = eval::<ty::Bool>(env, expr)?;
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

/// Matches a scrutinee to a expression pattern.
fn match_pattern<T: Evaluable>(env: &EnvRef, scrutinee: T::Output, pattern: Expr) -> EResult<bool> {
    match pattern {
        Expr::Variable(name) => env.set_var(name, T::to_value(scrutinee)).map(|_| true),
        Expr::Literal(lit_pattern) => T::match_literal(env, scrutinee, lit_pattern),
        Expr::Binary {
            operator,
            left,
            right,
        } => T::match_bin(env, scrutinee, operator, *left, *right),
        Expr::Call { function, args } => T::match_call(env, scrutinee, *function, args),
        _ => Err("pattern not supported".to_string()),
    }
}

/// Generic match-arm evaluation.
fn eval_match<T: Evaluable>(
    env: &EnvRef,
    arms: Vec<MatchArm>,
    scrutinee: Value,
) -> EResult<T::Output> {
    for MatchArm {
        pattern,
        guard,
        body,
    } in arms
    {
        let mut arm_env = env.child_scope();
        let matched = match_pattern::<ty::Any>(&mut arm_env, scrutinee.clone(), pattern)?;
        if !matched {
            continue;
        }

        if let Some(condition) = guard {
            if !eval::<ty::Bool>(&arm_env, condition)? {
                continue;
            }
        }
        return eval::<T>(&arm_env, body);
    }
    Err("no arm matched".to_string())
}

/// Evaluates a field access expression.
fn eval_field_access<T: Evaluable>(
    env: &EnvRef,
    object: String,
    field: String,
) -> EResult<T::Output> {
    let object_value = env.get_var(&object)?;
    let result = match object_value {
        Value::Graphic(g) => match g.ty {
            scene::GraphicType::Circle { radius } => match field.as_str() {
                // TEMP: 0 for x and y
                "x" => Value::Number(0.0),
                "y" => Value::Number(0.0),
                "radius" => Value::Number(radius),
                _ => return Err("unknown field".to_string()),
            },
            scene::GraphicType::Rect { width, height } => match field.as_str() {
                "x" => Value::Number(0.0),
                "y" => Value::Number(0.0),
                "width" => Value::Number(width),
                "height" => Value::Number(height),
                _ => return Err("unknown field".to_string()),
            },
            scene::GraphicType::Text { content } => match field.as_str() {
                "x" => Value::Number(0.0),
                "y" => Value::Number(0.0),
                "content" => Value::String(content),
                _ => return Err("unknown field".to_string()),
            },
            scene::GraphicType::Group { .. } => match field.as_str() {
                "x" => Value::Number(0.0),
                "y" => Value::Number(0.0),
                _ => return Err("unknown field".to_string()),
            },
        },
        _ => return Err("can not access field of this value".to_string()),
    };
    T::from_value(result)
}

// Controller for any type, branches to the appropriate functions of the runtime value
impl Evaluable for ty::Any {
    type Output = Value;
    fn to_value(value: Self::Output) -> Value {
        value
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        Ok(value)
    }
    fn eval_literal(env: &EnvRef, literal: Literal) -> EResult<Self::Output> {
        Ok(match literal {
            Literal::Number(n) => Value::Number(n),
            Literal::String(s) => Value::String(s),
            Literal::Bool(b) => Value::Bool(b),
            Literal::Color(_) => Value::Color(ty::Color::eval_literal(env, literal)?),
            Literal::List(_) => Value::List(ty::List::eval_literal(env, literal)?),
        })
    }
    fn match_literal(
        env: &EnvRef,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match scrutinee {
            Value::Number(s) => ty::Number::match_literal(env, s, literal_pattern),
            Value::String(s) => ty::String::match_literal(env, s, literal_pattern),
            Value::Bool(s) => ty::Bool::match_literal(env, s, literal_pattern),
            Value::Color(s) => ty::Color::match_literal(env, s, literal_pattern),
            Value::Graphic(s) => ty::Graphic::match_literal(env, s, literal_pattern),
            Value::List(s) => ty::List::match_literal(env, s, literal_pattern),
            Value::Function(s) => ty::Function::match_literal(env, s, literal_pattern),
        }
    }
    fn match_bin(
        env: &EnvRef,
        scrutinee: Self::Output,
        operator: op::Binary,
        left: Expr,
        right: Expr,
    ) -> EResult<bool> {
        match scrutinee {
            Value::Number(s) => ty::Number::match_bin(env, s, operator, left, right),
            Value::String(s) => ty::String::match_bin(env, s, operator, left, right),
            Value::Bool(s) => ty::Bool::match_bin(env, s, operator, left, right),
            Value::Color(s) => ty::Color::match_bin(env, s, operator, left, right),
            Value::Graphic(s) => ty::Graphic::match_bin(env, s, operator, left, right),
            Value::List(s) => ty::List::match_bin(env, s, operator, left, right),
            Value::Function(s) => ty::Function::match_bin(env, s, operator, left, right),
        }
    }
    fn match_call(
        env: &EnvRef,
        scrutinee: Self::Output,
        function: Expr,
        args: Vec<Expr>,
    ) -> EResult<bool> {
        match scrutinee {
            Value::Number(s) => ty::Number::match_call(env, s, function, args),
            Value::String(s) => ty::String::match_call(env, s, function, args),
            Value::Bool(s) => ty::Bool::match_call(env, s, function, args),
            Value::Color(s) => ty::Color::match_call(env, s, function, args),
            Value::Graphic(s) => ty::Graphic::match_call(env, s, function, args),
            Value::List(s) => ty::List::match_call(env, s, function, args),
            Value::Function(s) => ty::Function::match_call(env, s, function, args),
        }
    }
}
