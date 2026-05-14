use itertools::Itertools;

use crate::evaluator::expr::{Evaluable, match_pattern};
use crate::evaluator::{Context, EResult, Value, ty};
use crate::ir::ast::{Expr, Literal, Std, op};
use crate::ir::scene;

macro_rules! unpack_1 {
    ($iter:expr) => {
        $iter
            .into_iter()
            .exactly_one()
            .map_err(|_| "expected one argument")
    };
}
macro_rules! unpack_2 {
    ($iter:expr) => {
        $iter
            .into_iter()
            .collect_tuple::<(_, _)>()
            .ok_or("expected two arguments")
    };
}

impl Evaluable for ty::Graphic {
    type Output = scene::Graphic;
    fn to_value(value: Self::Output) -> Value {
        Value::Graphic(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Graphic(x) => Ok(x),
            _ => Err("expected a graphic".to_string()),
        }
    }
    fn eval_literal(_: &Context, _: Literal) -> EResult<Self::Output> {
        // No graphic literals, they are created through Std functions
        Err("expected a graphic".to_string())
    }
    fn match_literal(_: &mut Context, _: Self::Output, _: Literal) -> EResult<bool> {
        Err("pattern not supported".to_string())
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
    fn match_call(
        context: &mut Context,
        scrutinee: Self::Output,
        function: Expr,
        args: Vec<Expr>,
    ) -> EResult<bool> {
        let Expr::Std(func_pattern) = function else {
            return Err("pattern not supported".to_string());
        };

        match (scrutinee.ty, func_pattern) {
            (scene::GraphicType::Circle { radius }, Std::Circle) => {
                let radius_p = unpack_1!(args)?;
                match_pattern::<ty::Number>(context, radius, radius_p)
            }
            (scene::GraphicType::Rect { width, height }, Std::Rect) => {
                let (width_p, height_p) = unpack_2!(args)?;
                Ok(match_pattern::<ty::Number>(context, width, width_p)?
                    && match_pattern::<ty::Number>(context, height, height_p)?)
            }
            (scene::GraphicType::Text { content }, Std::Text) => {
                let content_p = unpack_1!(args)?;
                match_pattern::<ty::String>(context, content, content_p)
            }
            _ => Ok(false),
        }
    }
}
