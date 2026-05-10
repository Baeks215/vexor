use crate::evaluator::expr::{Evaluable, eval, match_pattern};
use crate::evaluator::{Context, EResult, Value, ty};
use crate::ir::ast::{self, Expr, Literal, op};
use crate::ir::scene;

impl Evaluable for ty::Graphic {
    type Output = scene::Graphic;
    fn to_value(value: Self::Output) -> Value {
        Value::Graphic(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Graphic(x) => Ok(x),
            _ => Err("Expected a graphic".to_string()),
        }
    }
    fn eval_literal(context: &Context, literal: Literal) -> EResult<Self::Output> {
        let Literal::Graphic(node) = literal else {
            return Err("Expected a graphic object".to_string());
        };
        Ok(scene::Graphic::new(match node {
            ast::Graphic::Circle { radius } => scene::GraphicType::Circle {
                radius: eval::<ty::Number>(context, *radius)?,
            },
            ast::Graphic::Rect { width, height } => scene::GraphicType::Rect {
                width: eval::<ty::Number>(context, *width)?,
                height: eval::<ty::Number>(context, *height)?,
            },
            ast::Graphic::Text { content } => scene::GraphicType::Text {
                content: eval::<ty::String>(context, *content)?,
            },
        }))
    }
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::Graphic(pattern) => Ok(match (scrutinee.ty, pattern) {
                (
                    scene::GraphicType::Circle { radius },
                    ast::Graphic::Circle { radius: radius_e },
                ) => match_pattern::<ty::Number>(context, radius, *radius_e)?,
                (
                    scene::GraphicType::Rect { width, height },
                    ast::Graphic::Rect {
                        width: width_e,
                        height: height_e,
                    },
                ) => {
                    match_pattern::<ty::Number>(context, width, *width_e)?
                        && match_pattern::<ty::Number>(context, height, *height_e)?
                }
                (
                    scene::GraphicType::Text { content },
                    ast::Graphic::Text { content: content_e },
                ) => match_pattern::<ty::String>(context, content, *content_e)?,
                _ => false,
            }),
            _ => Err("Expected a graphic literal".to_string()),
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
