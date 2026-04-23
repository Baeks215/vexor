//! Keyword parsers.

use super::{Input, lexeme};
use winnow::error::StrContext;
use winnow::{ModalResult, Parser};

/// Macro to define a set of keyword parsers
macro_rules! define_keywords {
    (
        // Comma-separated list of `parser_name => "keyword"`
        $($func_name:ident => $kw_str:expr),* $(,)?
    ) => {
        // Generates a parser function for every keyword in the list
        $(
            #[doc = concat!("Parses the `", $kw_str, "` keyword.")]
            pub fn $func_name<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
                lexeme($kw_str.context(StrContext::Label(concat!("keyword '", $kw_str, "'"))))
                    .parse_next(input)
            }
        )*

        /// Checks if a string is a keyword.
        pub fn is_keyword(s: &str) -> bool {
            matches!(s, $($kw_str)|*)
        }
    };
}

define_keywords! {
    // Defined keywords
    pk_let => "let",
    pk_export => "export",
    pk_fn => "fn",
    pk_where => "where",
    pk_match => "match",
    pk_if => "if",
    // Types
    pk_number => "number",
    pk_string => "string",
    pk_bool => "bool",
    pk_color => "color",
    pk_graphic => "graphic",
    // Primitives
    pk_circle => "circle",
    pk_rect => "rect",
    pk_text => "text",
    // Bool literals
    pk_true => "true",
    pk_false => "false",
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("let"));
        assert!(is_keyword("export"));
        assert!(is_keyword("color"));
        assert!(is_keyword("circle"));
        assert!(is_keyword("bool"));
        assert!(is_keyword("true"));
        assert!(is_keyword("false"));
    }

    #[test]
    fn test_keyword_parsers() {
        let mut input = Input::new("let  ");
        assert_eq!(pk_let.parse_next(&mut input).unwrap(), "let");
        assert_eq!(*input, "");

        let mut input = Input::new("circle\n");
        assert_eq!(pk_circle.parse_next(&mut input).unwrap(), "circle");
        assert_eq!(*input, "\n");

        let mut input = Input::new("not_a_keyword");
        assert!(pk_let.parse_next(&mut input).is_err());
    }
}
