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
    pk_let => "let",
    pk_color => "color",
    pk_export => "export",
}
