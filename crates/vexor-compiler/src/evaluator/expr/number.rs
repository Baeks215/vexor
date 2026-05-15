use crate::evaluator::expr::Evaluable;
use crate::evaluator::{EResult, EnvRef, Value, ty};
use crate::ir::Number;
use crate::ir::ast::{Expr, Literal, op};

impl Evaluable for ty::Number {
    type Output = Number;
    fn to_value(value: Self::Output) -> Value {
        Value::Number(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Number(x) => Ok(x),
            _ => Err("expected a number".to_string()),
        }
    }
    fn eval_literal(_: &EnvRef, literal: Literal) -> EResult<Self::Output> {
        match literal {
            Literal::Number(n) => Ok(n),
            _ => Err("expected a number".to_string()),
        }
    }
    fn match_literal(
        _: &EnvRef,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::Number(n) => Ok(scrutinee == n),
            _ => Err("expected a number literal".to_string()),
        }
    }
    fn match_bin(_: &EnvRef, _: Self::Output, _: op::Binary, _: Expr, _: Expr) -> EResult<bool> {
        Err("pattern not supported".to_string())
    }
    fn match_call(_: &EnvRef, _: Self::Output, _: Expr, _: Vec<Expr>) -> EResult<bool> {
        Err("pattern not supported".to_string())
    }
}
