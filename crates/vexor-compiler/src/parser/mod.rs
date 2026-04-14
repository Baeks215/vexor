//! Common parser utilities.

use winnow::ascii::{multispace0, space0};
use winnow::combinator::{delimited, terminated};
use winnow::error::ContextError;
use winnow::token::take_while;
use winnow::{LocatingSlice, ModalParser, ModalResult, Parser};

mod expr;
mod graphic;
mod keyword;
mod program;

pub use program::*;

/// Parser input type with location information.
type Input<'a> = LocatingSlice<&'a str>;

/// Combinator to discard whitespace after a parser
fn lexeme<'a, F, O>(inner: F) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    terminated(inner, space0)
}

/// Combinator to discard whitespace after a parser, including newlines.
fn ml_lexeme<'a, F, O>(inner: F) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    terminated(inner, multispace0)
}

/// Parse identifier without parsing whitespace after
fn p_identifier_no_ws<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    (
        take_while(1, |c: char| c.is_alphabetic() || c == '_'),
        take_while(0.., |c: char| c.is_alphanumeric() || c == '_'),
    )
        .take()
        .verify(|ident| !keyword::is_keyword(ident))
        .parse_next(input)
}

/// Parse identifier
fn p_identifier<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    lexeme(p_identifier_no_ws).parse_next(input)
}

// --- Helpers ---

/// Parse between brackets
fn bracketed<'a, F, O>(inner: F) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    delimited('(', inner, ')')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p_identifier() {
        let mut input = Input::new("foo_bar_123");
        assert_eq!(p_identifier.parse_next(&mut input).unwrap(), "foo_bar_123");
        assert_eq!(*input, "");

        let mut input = Input::new("_123 abc");
        assert_eq!(p_identifier.parse_next(&mut input).unwrap(), "_123");
        assert_eq!(*input, "abc");

        // Invalid identifier starts with a digit
        let mut input = Input::new("123");
        assert!(p_identifier.parse_next(&mut input).is_err());

        let mut input = Input::new("1abc");
        assert!(p_identifier.parse_next(&mut input).is_err());

        // Invalid identifier starts is a keyword
        let mut input = Input::new("let");
        assert!(p_identifier.parse_next(&mut input).is_err());

        let mut input = Input::new("color");
        assert!(p_identifier.parse_next(&mut input).is_err());

        // Valid identifier starts with keyword
        let mut input = Input::new("letabc");
        assert_eq!(p_identifier.parse_next(&mut input).unwrap(), "letabc");
        assert_eq!(*input, "");
    }

    #[test]
    fn test_lexeme() {
        let mut input = Input::new("foo  ");
        assert_eq!(lexeme("foo").parse_next(&mut input).unwrap(), "foo");
        assert_eq!(*input, "");

        let mut input = Input::new("foo\n\t ");
        assert_eq!(lexeme("foo").parse_next(&mut input).unwrap(), "foo");
        assert_eq!(*input, "\n\t ");
    }

    #[test]
    fn test_bracketed() {
        let mut input = Input::new("(foo)");
        assert_eq!(bracketed("foo").parse_next(&mut input).unwrap(), "foo");
        assert_eq!(*input, "");

        let mut input = Input::new("((foo))");
        assert_eq!(
            bracketed(bracketed("foo")).parse_next(&mut input).unwrap(),
            "foo"
        );
        assert_eq!(*input, "");
    }
}
