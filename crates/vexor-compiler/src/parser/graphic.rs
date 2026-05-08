//! Parser for graphic components

use crate::ir::ast;
use crate::parser::expr::p_expr;
use crate::parser::keyword::{self as k, pk_circle, pk_rect, pk_text};
use crate::parser::{Input, WhiteSpaceParser, braced, comma_list};
use winnow::combinator::{alt, separated_pair};
use winnow::error::{ContextError, ErrMode};
use winnow::{ModalResult, Parser};

macro_rules! extract_fields {
    ($fields:expr, [$($name:ident),+]) => {{
        let fields: Vec<(&str, _)> = $fields;

        // Check for unexpected
        let expected = [$(stringify!($name)),+];
        for (key, _) in fields.iter() {
            if !expected.contains(key) {
                // TODO: Return multiple errors for each unexpected field
                return Err(ErrMode::Cut(ContextError::new()));
            }
        }

        // Extract and move fields to tupl
        let mut map: std::collections::HashMap<&str, _> = fields.into_iter().collect();

        $(
            let $name = map.remove(stringify!($name))
                .ok_or_else(|| ErrMode::Cut(ContextError::new()))?;
        )+

        ($($name),+)
    }};
}

/// Parse an object literal
///   Contains fields with expr values
pub fn p_graphic<'a>(input: &mut Input<'a>) -> ModalResult<ast::Graphic> {
    let (name, fields) = (
        alt((pk_circle, pk_rect, pk_text)).ws(),
        braced(comma_list(
            0..,
            separated_pair(
                alt(("x", "y", "width", "height", "radius", "content", "color")),
                ':'.ws(),
                p_expr,
            ),
        )),
    )
        .ws()
        .parse_next(input)?;

    build_graphic(name, fields)
}

fn build_graphic(name: k::Graphic, fields: Vec<(&str, ast::Expr)>) -> ModalResult<ast::Graphic> {
    match name {
        k::Graphic::Circle => {
            let (x, y, radius, color) = extract_fields!(fields, [x, y, radius, color]);
            Ok(ast::Graphic::Circle {
                x: Box::new(x),
                y: Box::new(y),
                radius: Box::new(radius),
                color: Box::new(color),
            })
        }
        k::Graphic::Rect => {
            let (x, y, width, height, color) =
                extract_fields!(fields, [x, y, width, height, color]);
            Ok(ast::Graphic::Rect {
                x: Box::new(x),
                y: Box::new(y),
                width: Box::new(width),
                height: Box::new(height),
                color: Box::new(color),
            })
        }
        k::Graphic::Text => {
            let (x, y, content, color) = extract_fields!(fields, [x, y, content, color]);
            Ok(ast::Graphic::Text {
                x: Box::new(x),
                y: Box::new(y),
                content: Box::new(content),
                color: Box::new(color),
            })
        }
    }
}
