//! Parser for graphic components

use crate::ir::ast;
use crate::parser::expr::p_expr;
use crate::parser::keyword::{pk_circle, pk_rect, pk_text};
use crate::parser::{Input, WhiteSpaceParser, braced, p_raw_identifier_no_ws};
use winnow::combinator::{alt, separated, separated_pair};
use winnow::{ModalResult, Parser};

/// Parse a field name
fn p_field_name<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    p_raw_identifier_no_ws.ws().parse_next(input)
}

/// Parse an object literal
///   Contains fields with expr values
pub fn p_object<'a>(input: &mut Input<'a>) -> ModalResult<ast::Object> {
    (
        alt((pk_circle, pk_rect, pk_text)).ws().map(str::to_string), // Object keywords
        braced(separated(
            0..,
            separated_pair(p_field_name.map(str::to_string), ':'.ws(), p_expr),
            ','.mws(),
        )),
    )
        .ws()
        .map(|(name, fields)| ast::Object { name, fields })
        .parse_next(input)
}
