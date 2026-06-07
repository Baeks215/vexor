//! Evaluator for expressions

use std::rc::Rc;

use crate::evaluator::expr::constants::get_constant;
use crate::evaluator::{EResult, EnvExt, EnvRef, WithSpan};
use crate::ir::ast::{Expr, Literal, SpanExpr};

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
pub fn eval<T: Evaluable>(env: &EnvRef, expr: &SpanExpr) -> EResult<T::Output> {
    let result: EResult<T::Output> = match &expr.node {
        Expr::Literal(literal) => eval_literal::<T>(env, literal),
        Expr::Variable(name) => env.get_var(name).and_then(T::expect),
        Expr::Const(c) => T::expect(get_constant(*c)),
        Expr::Call { function, args } => {
            let function = eval::<ty::Callable>(env, function).with_span_if_missing(&expr.span)?;
            let args: Vec<Value> = args
                .iter()
                .map(|arg_expr| eval::<ty::Any>(env, arg_expr))
                .collect::<Result<Vec<_>, _>>()
                .with_span_if_missing(&expr.span)?;

            eval_call::<T, _>(env, function, args)
        }
        Expr::Function(func) => {
            T::expect(Value::from(Callable::User {
                func: Rc::new(func.clone()),
                // Capture the current environment
                closure_env: env.clone(), // Cloned reference
            }))
        }
        Expr::Std(func) => T::expect(Value::from(Callable::Std(*func))),
        Expr::Binary {
            operator,
            left,
            right,
        } => eval_op_bin::<T>(env, *operator, left, right),
        Expr::Unary { operator, operand } => eval_op_un::<T>(env, *operator, operand),
        Expr::Match { scrutinee, arms } => {
            let s = eval::<ty::Any>(env, scrutinee)?;
            eval_match::<T>(env, arms, s)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            if eval::<ty::Bool>(env, condition)? {
                eval::<T>(env, then_branch)
            } else {
                eval::<T>(env, else_branch)
            }
        }
    };
    result.with_span_if_missing(&expr.span)
}

/// Evaluates a literal expression
fn eval_literal<T: Evaluable>(env: &EnvRef, literal: &Literal) -> EResult<T::Output> {
    let result = match literal {
        Literal::Number(n) => Value::Number(*n),
        Literal::String(s) => Value::String(s.clone()),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::List(l) => Value::List(list::eval_literal(env, l)?),
        Literal::Tuple(exprs) => {
            let values: Box<[Value]> = exprs
                .iter()
                .map(|e| eval::<ty::Any>(env, e))
                .collect::<Result<_, _>>()?;
            Value::Tuple(values)
        }
    };
    T::expect(result)
}
