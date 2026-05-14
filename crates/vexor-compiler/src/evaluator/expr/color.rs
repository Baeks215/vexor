use crate::evaluator::expr::{Evaluable, eval, match_pattern};
use crate::evaluator::{EResult, EnvRef, Value, ty};
use crate::ir::ast::{self, Expr, Literal, op};
use crate::ir::scene;

impl Evaluable for ty::Color {
    type Output = scene::Color;
    fn to_value(value: Self::Output) -> Value {
        Value::Color(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Color(x) => Ok(x),
            _ => Err("expected a color".to_string()),
        }
    }
    fn eval_literal(env: &EnvRef, literal: Literal) -> EResult<Self::Output> {
        let Literal::Color(ast::Color::Rgba { r, g, b, a }) = literal else {
            return Err("expected a color".to_string());
        };
        Ok(scene::Color::Rgba {
            r: eval::<ty::Number>(env, *r)?,
            g: eval::<ty::Number>(env, *g)?,
            b: eval::<ty::Number>(env, *b)?,
            a: eval::<ty::Number>(env, *a)?,
        })
    }
    fn match_literal(
        env: &EnvRef,
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
                Ok(match_pattern::<ty::Number>(env, r, *r_expr)?
                    && match_pattern::<ty::Number>(env, g, *g_expr)?
                    && match_pattern::<ty::Number>(env, b, *b_expr)?
                    && match_pattern::<ty::Number>(env, a, *a_expr)?)
            }
            _ => Err("expected a color literal".to_string()),
        }
    }
    fn match_bin(_: &EnvRef, _: Self::Output, _: op::Binary, _: Expr, _: Expr) -> EResult<bool> {
        Err("pattern not supported".to_string())
    }
    fn match_call(_: &EnvRef, _: Self::Output, _: Expr, _: Vec<Expr>) -> EResult<bool> {
        Err("pattern not supported".to_string())
    }
}
