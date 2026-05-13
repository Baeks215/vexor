use crate::evaluator::expr::Evaluable;
use crate::evaluator::{Context, EResult, Value, ty};
use crate::ir::ast::{self, Expr, Literal, op};

impl Evaluable for ty::Function {
    type Output = ast::Function;
    fn to_value(value: Self::Output) -> Value {
        Value::Function(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Function(f) => Ok(f),
            _ => Err("expected a function".to_string()),
        }
    }
    fn eval_literal(_: &Context, literal: Literal) -> EResult<Self::Output> {
        match literal {
            Literal::Function(f) => Ok(f),
            _ => Err("expected a function".to_string()),
        }
    }
    fn match_literal(_: &mut Context, _: Self::Output, _: Literal) -> EResult<bool> {
        Err("cannot pattern match a function".to_string())
    }
    fn match_bin(
        _: &mut Context,
        _: Self::Output,
        _: op::Binary,
        _: Expr,
        _: Expr,
    ) -> EResult<bool> {
        Err("cannot pattern match a function".to_string())
    }
}
