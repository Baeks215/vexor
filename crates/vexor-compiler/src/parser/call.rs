use itertools::Itertools;
use winnow::combinator::{alt, cut_err, peek, terminated};
use winnow::{ModalResult, Parser};

use crate::ir::ast;
use crate::parser::{Input, ParserExt, expected, keyword as k};
use crate::parser::{bracketed, comma_list, expr::p_expr, p_identifier};

/// Parses a function call.
pub fn p_call<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    (p_identifier, bracketed(comma_list(0.., p_expr)))
        .ws()
        .map(|(function, args)| ast::Expr::Call {
            function: function.to_string(),
            args,
        })
        .parse_next(input)
}

/// Parses a standard function call.
pub fn p_std<'a>(input: &mut Input<'a>) -> ModalResult<ast::Std> {
    let (function, args) = (
        terminated(
            alt((
                alt((k::pk_rad, k::pk_sin, k::pk_cos, k::pk_tan)),
                k::pk_map,
                alt((
                    k::pk_move,
                    k::pk_scale,
                    k::pk_rotate,
                    k::pk_fill,
                    k::pk_stroke,
                )),
            )),
            // Required to disregard partial matches like `filled`
            peek('('),
        ),
        cut_err(bracketed(comma_list(0.., p_expr))),
    )
        .ws()
        .parse_next(input)?;

    build_std(function, args).map_err(|e| expected(e, input))
}

/// Parse an object literal
///   Contains fields with expr values
pub fn p_graphic<'a>(input: &mut Input<'a>) -> ModalResult<ast::Graphic> {
    let (function, args) = (
        terminated(
            alt((k::pk_circle, k::pk_rect, k::pk_text, k::pk_group)),
            peek('('),
        ),
        cut_err(bracketed(comma_list(0.., p_expr))),
    )
        .ws()
        .parse_next(input)?;

    build_graphic(function, args).map_err(|e| expected(e, input))
}

macro_rules! unpack_1 {
    ($iter:expr) => {
        $iter
            .into_iter()
            .exactly_one()
            .map_err(|_| "exactly one argument")
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
macro_rules! unpack_3 {
    ($iter:expr) => {
        $iter
            .into_iter()
            .collect_tuple::<(_, _, _)>()
            .ok_or("expected three arguments")
    };
}

type StrResult<T> = Result<T, &'static str>;

/// Build a standard function call
fn build_std(function: k::Std, args: Vec<ast::Expr>) -> StrResult<ast::Std> {
    Ok(match function {
        k::Std::Rad => ast::Std::Rad(Box::new(unpack_1!(args)?)),
        k::Std::Sin => ast::Std::Sin(Box::new(unpack_1!(args)?)),
        k::Std::Cos => ast::Std::Cos(Box::new(unpack_1!(args)?)),
        k::Std::Tan => ast::Std::Tan(Box::new(unpack_1!(args)?)),
        k::Std::Map => {
            let (function, list) = unpack_2!(args)?;
            ast::Std::Map {
                function: Box::new(function),
                list: Box::new(list),
            }
        }
        k::Std::Move => {
            let (x, y, graphic) = unpack_3!(args)?;
            ast::Std::Move {
                x: Box::new(x),
                y: Box::new(y),
                graphic: Box::new(graphic),
            }
        }
        k::Std::Scale => {
            let (scale, graphic) = unpack_2!(args)?;
            ast::Std::Scale {
                scale: Box::new(scale),
                graphic: Box::new(graphic),
            }
        }
        k::Std::Rotate => {
            let (angle, graphic) = unpack_2!(args)?;
            ast::Std::Rotate {
                angle: Box::new(angle),
                graphic: Box::new(graphic),
            }
        }
        k::Std::Fill => {
            let (color, graphic) = unpack_2!(args)?;
            ast::Std::Fill {
                color: Box::new(color),
                graphic: Box::new(graphic),
            }
        }
        k::Std::Stroke => {
            let (width, color, graphic) = unpack_3!(args)?;
            ast::Std::Stroke {
                width: Box::new(width),
                color: Box::new(color),
                graphic: Box::new(graphic),
            }
        }
    })
}

/// Build a graphic literal call
fn build_graphic(function: k::Graphic, args: Vec<ast::Expr>) -> StrResult<ast::Graphic> {
    match function {
        k::Graphic::Circle => {
            let radius = unpack_1!(args)?;
            Ok(ast::Graphic::Circle {
                radius: Box::new(radius),
            })
        }
        k::Graphic::Rect => {
            let (width, height) = unpack_2!(args)?;
            Ok(ast::Graphic::Rect {
                width: Box::new(width),
                height: Box::new(height),
            })
        }
        k::Graphic::Text => {
            let content = unpack_1!(args)?;
            Ok(ast::Graphic::Text {
                content: Box::new(content),
            })
        }
        k::Graphic::Group => {
            let children = unpack_1!(args)?;
            Ok(ast::Graphic::Group {
                children: Box::new(children),
            })
        }
    }
}
