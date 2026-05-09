use crate::evaluator::expr::{Evaluable, eval, match_pattern};
use crate::evaluator::{Context, EResult, Value};
use crate::ir::ast::{self, Expr, Literal, op};
use crate::ir::scene;
use crate::ir::scene::marker;

impl Evaluable for marker::Graphic {
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
        match node {
            ast::Graphic::Circle {
                x,
                y,
                radius,
                color,
            } => Ok(scene::Graphic::Circle {
                x: eval::<marker::Number>(context, *x)?,
                y: eval::<marker::Number>(context, *y)?,
                radius: eval::<marker::Number>(context, *radius)?,
                color: eval::<marker::Color>(context, *color)?,
            }),
            ast::Graphic::Rect {
                x,
                y,
                width,
                height,
                color,
            } => Ok(scene::Graphic::Rect {
                x: eval::<marker::Number>(context, *x)?,
                y: eval::<marker::Number>(context, *y)?,
                width: eval::<marker::Number>(context, *width)?,
                height: eval::<marker::Number>(context, *height)?,
                color: eval::<marker::Color>(context, *color)?,
            }),
            ast::Graphic::Text {
                x,
                y,
                content,
                color,
            } => Ok(scene::Graphic::Text {
                x: eval::<marker::Number>(context, *x)?,
                y: eval::<marker::Number>(context, *y)?,
                content: eval::<marker::String>(context, *content)?,
                color: eval::<marker::Color>(context, *color)?,
            }),
        }
    }
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::Graphic(pattern) => match (scrutinee, pattern) {
                (
                    scene::Graphic::Circle {
                        x,
                        y,
                        radius,
                        color,
                    },
                    ast::Graphic::Circle {
                        x: x_e,
                        y: y_e,
                        radius: radius_e,
                        color: color_e,
                    },
                ) => Ok(match_pattern::<marker::Number>(context, x, *x_e)?
                    && match_pattern::<marker::Number>(context, y, *y_e)?
                    && match_pattern::<marker::Number>(context, radius, *radius_e)?
                    && match_pattern::<marker::Color>(context, color, *color_e)?),
                (
                    scene::Graphic::Rect {
                        x,
                        y,
                        width,
                        height,
                        color,
                    },
                    ast::Graphic::Rect {
                        x: x_e,
                        y: y_e,
                        width: width_e,
                        height: height_e,
                        color: color_e,
                    },
                ) => Ok(match_pattern::<marker::Number>(context, x, *x_e)?
                    && match_pattern::<marker::Number>(context, y, *y_e)?
                    && match_pattern::<marker::Number>(context, width, *width_e)?
                    && match_pattern::<marker::Number>(context, height, *height_e)?
                    && match_pattern::<marker::Color>(context, color, *color_e)?),
                (
                    scene::Graphic::Text {
                        x,
                        y,
                        content,
                        color,
                    },
                    ast::Graphic::Text {
                        x: x_e,
                        y: y_e,
                        content: content_e,
                        color: color_e,
                    },
                ) => Ok(match_pattern::<marker::Number>(context, x, *x_e)?
                    && match_pattern::<marker::Number>(context, y, *y_e)?
                    && match_pattern::<marker::String>(context, content, *content_e)?
                    && match_pattern::<marker::Color>(context, color, *color_e)?),
                _ => Ok(false),
            },
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
