//! Common parser utilities.

use crate::parser::common::keyword::is_keyword;
use winnow::ascii::multispace0;
use winnow::combinator::{delimited, terminated};
use winnow::error::ContextError;
use winnow::token::take_while;
use winnow::{LocatingSlice, ModalParser, ModalResult, Parser};

pub mod keyword;

/// Parser input type with location information.
pub type Input<'a> = LocatingSlice<&'a str>;

/// Combinator to discard whitespace after a parser
pub fn lexeme<'a, F, O>(inner: F) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    terminated(inner, multispace0)
}

/// Parse identifier
pub fn p_identifier<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    lexeme(
        (
            take_while(1, |c: char| c.is_alphabetic() || c == '_'),
            take_while(0.., |c: char| c.is_alphanumeric() || c == '_'),
        )
            .take(),
    )
    .verify(|ident| !is_keyword(ident))
    .parse_next(input)
}

// --- Helpers ---

/// Parse between brackets
pub fn bracketed<'a, F, O>(inner: F) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    delimited('(', inner, ')')
}
