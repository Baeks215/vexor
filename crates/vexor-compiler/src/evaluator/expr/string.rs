use crate::evaluator::expr::Evaluable;
use crate::evaluator::{Context, EResult, Value, ty};
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
    fn eval_literal(_: &Context, literal: Literal) -> EResult<Self::Output> {
        match literal {
            Literal::String(s) => Ok(s),
            _ => Err("expected a string".to_string()),
        }
    }
    fn match_literal(
        _: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::String(s) => Ok(scrutinee == s),
            _ => Err("expected a string literal".to_string()),
        }
    }
    fn match_bin(
        _: &mut Context,
        _: Self::Output,
        _: op::Binary,
        _: Expr,
        _: Expr,
    ) -> EResult<bool> {
        Err("pattern not supported".to_string())
    }
}
