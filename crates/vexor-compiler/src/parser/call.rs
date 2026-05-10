use itertools::Itertools;
use winnow::combinator::alt;
use winnow::error::{ContextError, ErrMode};
use winnow::{ModalResult, Parser};

use crate::ir::ast;
use crate::parser::{Input, WhiteSpaceParser, keyword as k};
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
        alt((k::pk_rad, k::pk_sin, k::pk_cos, k::pk_tan, k::pk_map)),
        bracketed(comma_list(0.., p_expr)),
    )
        .ws()
        .parse_next(input)?;

    build_std(function, args)
}

/// Parse an object literal
///   Contains fields with expr values
pub fn p_graphic<'a>(input: &mut Input<'a>) -> ModalResult<ast::Graphic> {
    let (function, args) = (
        alt((k::pk_circle, k::pk_rect, k::pk_text)).ws(),
        bracketed(comma_list(0.., p_expr)),
    )
        .ws()
        .parse_next(input)?;

    build_graphic(function, args)
}

macro_rules! unpack_one {
    ($iter:expr) => {
        $iter
            .into_iter()
            .exactly_one()
            .map_err(|_| ErrMode::Cut(ContextError::new()))
    };
}
macro_rules! unpack {
    ($iter:expr) => {
        $iter
            .into_iter()
            .collect_tuple()
            .ok_or(ErrMode::Cut(ContextError::new()))
    };
}

/// Build a standard function call
fn build_std(function: k::Std, args: Vec<ast::Expr>) -> ModalResult<ast::Std> {
    Ok(match function {
        k::Std::Rad => ast::Std::Rad(Box::new(unpack_one!(args)?)),
        k::Std::Sin => ast::Std::Sin(Box::new(unpack_one!(args)?)),
        k::Std::Cos => ast::Std::Cos(Box::new(unpack_one!(args)?)),
        k::Std::Tan => ast::Std::Tan(Box::new(unpack_one!(args)?)),
        k::Std::Map => {
            let (function, list) = unpack!(args)?;
            ast::Std::Map {
                function: Box::new(function),
                list: Box::new(list),
            }
        }
    })
}

/// Build a graphic literal call
fn build_graphic(function: k::Graphic, args: Vec<ast::Expr>) -> ModalResult<ast::Graphic> {
    match function {
        k::Graphic::Circle => {
            let radius = unpack_one!(args)?;
            Ok(ast::Graphic::Circle {
                radius: Box::new(radius),
            })
        }
        k::Graphic::Rect => {
            let (width, height) = unpack!(args)?;
            Ok(ast::Graphic::Rect {
                width: Box::new(width),
                height: Box::new(height),
            })
        }
        k::Graphic::Text => {
            let content = unpack_one!(args)?;
            Ok(ast::Graphic::Text {
                content: Box::new(content),
            })
        }
    }
}
