use crate::evaluator::expr::{Evaluable, eval, match_pattern};
use crate::evaluator::{Context, EResult, Value, ty};
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
    fn eval_literal(context: &Context, literal: Literal) -> EResult<Self::Output> {
        let Literal::Color(ast::Color::Rgba { r, g, b, a }) = literal else {
            return Err("expected a color".to_string());
        };
        Ok(scene::Color::Rgba {
            r: eval::<ty::Number>(context, *r)?,
            g: eval::<ty::Number>(context, *g)?,
            b: eval::<ty::Number>(context, *b)?,
            a: eval::<ty::Number>(context, *a)?,
        })
    }
    fn match_literal(
        context: &mut Context,
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
                Ok(match_pattern::<ty::Number>(context, r, *r_expr)?
                    && match_pattern::<ty::Number>(context, g, *g_expr)?
                    && match_pattern::<ty::Number>(context, b, *b_expr)?
                    && match_pattern::<ty::Number>(context, a, *a_expr)?)
            }
            _ => Err("expected a color literal".to_string()),
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
