//! Evaluator for expressions

use crate::evaluator::expr::constants::get_constant;
use crate::evaluator::{EResult, EnvExt, EnvRef, WithSpan};
use crate::ir::ast::{Expr, Literal, SpanExpr};
use crate::ir::scene;

mod constants;
mod function;
mod list;
mod matcher;
mod operator;
mod types;

pub use function::Callable;
pub use types::*;

use function::eval_call;
use matcher::eval_match;
use operator::{eval_op_bin, eval_op_un};

/// Evaluates an expression and returns the result as the expected output type.
pub fn eval<T: Evaluable>(env: &EnvRef, expr: SpanExpr) -> EResult<T::Output> {
    let span = expr.span.clone();
    let result: EResult<T::Output> = match expr.node {
        Expr::Literal(literal) => eval_literal::<T>(env, literal),
        Expr::Variable(name) => {
            let value = env.get_var(&name)?;
            T::expect(value)
        }
        Expr::Const(c) => T::expect(get_constant(c)),
        Expr::Call { function, args } => {
            let function = eval::<ty::Callable>(env, *function)?;
            let args: Vec<Value> = args
                .into_iter()
                .map(|arg_expr| eval::<ty::Any>(env, arg_expr))
                .collect::<Result<Vec<_>, _>>()?;

            eval_call::<T>(env, function, args)
        }
        Expr::Function(func) => {
            T::expect(Value::from(Callable::User {
                func,
                // Capture the current environment
                closure_env: env.clone(), // Cloned reference
            }))
        }
        Expr::Std(func) => T::expect(Value::from(Callable::Std(func))),
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
        Expr::Field { object, field } => eval_field_access::<T>(env, *object, field),
    };
    result.with_span_if_missing(span)
}

/// Evaluates a literal expression
fn eval_literal<T: Evaluable>(env: &EnvRef, literal: Literal) -> EResult<T::Output> {
    let result = match literal {
        Literal::Number(n) => Value::Number(n),
        Literal::String(s) => Value::String(s),
        Literal::Bool(b) => Value::Bool(b),
        Literal::List(l) => Value::List(list::eval_literal(env, l)?),
        Literal::Tuple(exprs) => {
            let values: Box<[Value]> = exprs
                .into_iter()
                .map(|e| eval::<ty::Any>(env, e))
                .collect::<Result<_, _>>()?;
            Value::Tuple(values)
        }
    };
    T::expect(result)
}

/// Evaluates a field access expression.
fn eval_field_access<T: Evaluable>(
    env: &EnvRef,
    object: SpanExpr,
    field: String,
) -> EResult<T::Output> {
    let object_value = eval::<ty::Any>(env, object)?;
    let result = match object_value {
        Value::Graphic(g) => match g.ty {
            scene::GraphicType::Circle { radius } => match field.as_str() {
                // TEMP: 0 for x and y
                "x" => Value::from(0.0),
                "y" => Value::from(0.0),
                "radius" => Value::from(radius),
                _ => return Err("unknown field".into()),
            },
            scene::GraphicType::Rect { width, height } => match field.as_str() {
                "x" => Value::from(0.0),
                "y" => Value::from(0.0),
                "width" => Value::from(width),
                "height" => Value::from(height),
                _ => return Err("unknown field".into()),
            },
            scene::GraphicType::Text { content } => match field.as_str() {
                "x" => Value::from(0.0),
                "y" => Value::from(0.0),
                "content" => Value::from(content),
                _ => return Err("unknown field".into()),
            },
            scene::GraphicType::Group { .. } => match field.as_str() {
                "x" => Value::from(0.0),
                "y" => Value::from(0.0),
                _ => return Err("unknown field".into()),
            },
            scene::GraphicType::Path { .. } => {
                return Err("cannot access fields of a path".into());
            }
        },
        _ => return Err("can not access field of this value".into()),
    };
    T::expect(result)
}
