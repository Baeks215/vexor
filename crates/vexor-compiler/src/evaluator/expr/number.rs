use crate::evaluator::expr::Evaluable;
use crate::evaluator::{Context, EResult, Value};
use crate::ir::Number;
use crate::ir::ast::{Expr, Literal, op};
use crate::ir::scene::marker;

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
