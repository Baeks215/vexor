use crate::evaluator::expr::Evaluable;
use crate::evaluator::{EResult, EnvRef, Value, ty};
use crate::ir::ast::{Expr, Literal, op};

impl Evaluable for ty::String {
    type Output = String;
    fn to_value(value: Self::Output) -> Value {
        Value::String(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err("expected a string".to_string()),
        }
    }
    fn eval_literal(_: &EnvRef, literal: Literal) -> EResult<Self::Output> {
        match literal {
            Literal::String(s) => Ok(s),
            _ => Err("expected a string".to_string()),
        }
    }
    fn match_literal(
        _: &EnvRef,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::String(s) => Ok(scrutinee == s),
            _ => Err("expected a string literal".to_string()),
        }
    }
    fn match_bin(_: &EnvRef, _: Self::Output, _: op::Binary, _: Expr, _: Expr) -> EResult<bool> {
        Err("pattern not supported".to_string())
    }
    fn match_call(_: &EnvRef, _: Self::Output, _: Expr, _: Vec<Expr>) -> EResult<bool> {
        Err("pattern not supported".to_string())
    }
}
