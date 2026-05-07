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

trait WhiteSpaceParser<'a, O>: ModalParser<Input<'a>, O, ContextError> {
    /// Discard whitespace after the parser.
    fn ws(self) -> impl ModalParser<Input<'a>, O, ContextError>;
    /// Discard whitespace after the parser, including newlines.
    fn mws(self) -> impl ModalParser<Input<'a>, O, ContextError>;
}

impl<'a, O, P> WhiteSpaceParser<'a, O> for P
where
    P: ModalParser<Input<'a>, O, ContextError>,
{
    fn ws(self) -> impl ModalParser<Input<'a>, O, ContextError> {
        terminated(self, space0)
    }

    fn mws(self) -> impl ModalParser<Input<'a>, O, ContextError> {
        terminated(self, multispace0)
    }
}

/// Parse identifier, no ws
fn p_identifier<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    (
        take_while(1, |c: char| c.is_alphabetic() || c == '_'),
        take_while(0.., |c: char| c.is_alphanumeric() || c == '_'),
    )
        .take()
        // Ensure the identifier is not a keyword
        .verify(|ident| !keyword::is_keyword(ident))
        .parse_next(input)
}

// --- Helpers ---

/// Parse between brackets "()"
fn bracketed<'a, F, O>(inner: F) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    delimited(('(', space0), inner, (space0, ')'))
}

/// Parse between braces "{}"
///   Can contain new lines within braces
fn braced<'a, F, O>(inner: F) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    delimited(('{', multispace0), inner, (multispace0, '}'))
}

/// Parse between braces "[]"
///   Can contain new lines within braces
fn square_braced<'a, F, O>(inner: F) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    delimited(('[', multispace0), inner, (multispace0, ']'))
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
        assert_eq!(*input, " abc");

        // Invalid identifier starts with a digit
        let mut input = Input::new("123");
        assert!(p_identifier.parse_next(&mut input).is_err());

        let mut input = Input::new("1abc");
        assert!(p_identifier.parse_next(&mut input).is_err());

        // Invalid identifier starts is a keyword
        let mut input = Input::new("let");
        assert!(p_identifier.parse_next(&mut input).is_err());

        let mut input = Input::new("export");
        assert!(p_identifier.parse_next(&mut input).is_err());

        // Valid identifier starts with keyword
        let mut input = Input::new("letabc");
        assert_eq!(p_identifier.parse_next(&mut input).unwrap(), "letabc");
        assert_eq!(*input, "");
    }

    #[test]
    fn test_lexeme() {
        let mut input = Input::new("foo  ");
        assert_eq!("foo".ws().parse_next(&mut input).unwrap(), "foo");
        assert_eq!(*input, "");

        let mut input = Input::new("foo\n\t ");
        assert_eq!("foo".ws().parse_next(&mut input).unwrap(), "foo");
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

        // Whitespace inside brackets
        let mut input = Input::new("( foo )");
        assert_eq!(bracketed("foo").parse_next(&mut input).unwrap(), "foo");
        assert_eq!(*input, "");

        let mut input = Input::new("( ( foo ) )");
        assert_eq!(
            bracketed(bracketed("foo")).parse_next(&mut input).unwrap(),
            "foo"
        );
        assert_eq!(*input, "");
    }

    #[test]
    fn test_braced() {
        let mut input = Input::new("{foo}");
        assert_eq!(braced("foo").parse_next(&mut input).unwrap(), "foo");
        assert_eq!(*input, "");

        // Whitespace inside braces
        let mut input = Input::new("{ foo }");
        assert_eq!(braced("foo").parse_next(&mut input).unwrap(), "foo");
        assert_eq!(*input, "");

        // Newlines inside braces
        let mut input = Input::new("{\n  foo\n}");
        assert_eq!(braced("foo").parse_next(&mut input).unwrap(), "foo");
        assert_eq!(*input, "");

        // braced does NOT consume trailing whitespace after `}`
        let mut input = Input::new("{foo}\n");
        assert_eq!(braced("foo").parse_next(&mut input).unwrap(), "foo");
        assert_eq!(*input, "\n");
    }
}
