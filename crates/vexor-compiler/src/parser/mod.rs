//! Common parser utilities.

use winnow::ascii::{line_ending, multispace1, space1, till_line_ending};
use winnow::combinator::{alt, cut_err, delimited, preceded, repeat, separated, terminated};
use winnow::error::{AddContext, ContextError, ErrMode, StrContext, StrContextValue};
use winnow::stream::Stream;
use winnow::stream::{Accumulate, Range};
use winnow::{LocatingSlice, ModalParser, ModalResult, Parser};

mod expr;
mod function;
mod keyword;
mod program;

pub use program::parse_program;

/// Parser input type with location information.
type Input<'a> = LocatingSlice<&'a str>;

trait ParserExt<'a, O>: ModalParser<Input<'a>, O, ContextError> {
    /// Discard whitespace after the parser.
    fn ws(self) -> impl ModalParser<Input<'a>, O, ContextError>;
    /// Discard whitespace after the parser, including newlines.
    fn mws(self) -> impl ModalParser<Input<'a>, O, ContextError>;
    /// Add context error label to the parser.
    fn label(self, label: &'static str) -> impl ModalParser<Input<'a>, O, ContextError>;
    /// Add context error expected description to the parser.
    fn expected(self, description: &'static str) -> impl ModalParser<Input<'a>, O, ContextError>;
    /// Add context error expected string literal to the parser.
    fn expected_lit(self, literal: &'static str) -> impl ModalParser<Input<'a>, O, ContextError>;
    /// Add context error expected char literal to the parser.
    fn expected_char(self, char_literal: char) -> impl ModalParser<Input<'a>, O, ContextError>;
}

impl<'a, O, P> ParserExt<'a, O> for P
where
    P: ModalParser<Input<'a>, O, ContextError>,
{
    fn ws(self) -> impl ModalParser<Input<'a>, O, ContextError> {
        terminated(self, p_ws)
    }
    fn mws(self) -> impl ModalParser<Input<'a>, O, ContextError> {
        terminated(self, p_mws)
    }
    fn label(self, label: &'static str) -> impl ModalParser<Input<'a>, O, ContextError> {
        self.context(StrContext::Label(label))
    }
    fn expected(self, description: &'static str) -> impl ModalParser<Input<'a>, O, ContextError> {
        self.context(StrContext::Expected(StrContextValue::Description(
            description,
        )))
    }
    fn expected_lit(self, literal: &'static str) -> impl ModalParser<Input<'a>, O, ContextError> {
        self.context(StrContext::Expected(StrContextValue::StringLiteral(
            literal,
        )))
    }
    fn expected_char(self, char_literal: char) -> impl ModalParser<Input<'a>, O, ContextError> {
        self.context(StrContext::Expected(StrContextValue::CharLiteral(
            char_literal,
        )))
    }
}

/// Parse whitespace, and exclude comments
fn p_ws<'a>(input: &mut Input<'a>) -> ModalResult<()> {
    repeat(0.., alt((space1.void(), p_line_comment))).parse_next(input)
}

/// Parse whitespace, including newlines, and exclude comments
fn p_mws<'a>(input: &mut Input<'a>) -> ModalResult<()> {
    repeat(0.., alt((multispace1.void(), p_line_comment))).parse_next(input)
}

/// Parse at least 1 new line
fn newline1<'a>(input: &mut Input<'a>) -> ModalResult<()> {
    repeat(1.., line_ending.ws())
        .map(|_: ()| ())
        .parse_next(input)
}

/// Parse line comment
fn p_line_comment<'a>(input: &mut Input<'a>) -> ModalResult<()> {
    ("--", till_line_ending).void().parse_next(input)
}

// --- Helpers ---

/// Parse a char literal with expected context
fn exp_char<'a>(lit: char) -> impl ModalParser<Input<'a>, char, ContextError> {
    lit.expected_char(lit)
}

/// Parse a string literal with expected context
fn exp_string<'a>(lit: &'static str) -> impl ModalParser<Input<'a>, &'a str, ContextError> {
    lit.expected_lit(lit)
}

/// Parse between char delimiters
///   Can be across multiple lines
fn delim<'a, F, O>(
    left: char,
    inner: F,
    right: char,
) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    delimited((exp_char(left), p_mws), inner, (p_mws, exp_char(right)))
}

/// Parse between char delimiters
///   Commits if the left delimiter is found
///   Can be across multiple lines
fn delim_cut<'a, F, O>(
    left: char,
    inner: F,
    right: char,
) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
{
    preceded(
        (exp_char(left), p_mws),
        cut_err(terminated(inner, (p_mws, exp_char(right)))),
    )
}

/// Parse a comma-separated list of items
///   Can be across multiple lines
fn comma_list<'a, F, O, Accumulator>(
    occurrences: impl Into<Range>,
    inner: F,
) -> impl ModalParser<Input<'a>, Accumulator, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
    Accumulator: Accumulate<O>,
{
    separated(occurrences, inner, (p_mws, exp_char(','), p_mws))
}

/// Created context error for expected input
fn expected(desc: &'static str, input: &mut Input<'_>) -> ErrMode<ContextError> {
    ErrMode::Cut(ContextError::new().add_context(
        input,
        &input.checkpoint(),
        StrContext::Expected(StrContextValue::Description(desc)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_delim() {
        let mut input = Input::new("(foo)");
        assert_eq!(
            delim('(', "foo", ')').parse_next(&mut input).unwrap(),
            "foo"
        );
        assert_eq!(*input, "");

        let mut input = Input::new("((foo))");
        assert_eq!(
            delim('(', delim('(', "foo", ')'), ')')
                .parse_next(&mut input)
                .unwrap(),
            "foo"
        );
        assert_eq!(*input, "");

        // Whitespace inside brackets
        let mut input = Input::new("( foo )");
        assert_eq!(
            delim('(', "foo", ')').parse_next(&mut input).unwrap(),
            "foo"
        );
        assert_eq!(*input, "");

        let mut input = Input::new("( ( foo ) )");
        assert_eq!(
            delim('(', delim('(', "foo", ')'), ')')
                .parse_next(&mut input)
                .unwrap(),
            "foo"
        );
        assert_eq!(*input, "");
    }
}
