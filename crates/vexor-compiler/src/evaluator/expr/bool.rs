use crate::evaluator::expr::Evaluable;
use crate::evaluator::{Context, EResult, Value, ty};
use crate::ir::ast::{Expr, Literal, op};

impl Evaluable for ty::Bool {
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
