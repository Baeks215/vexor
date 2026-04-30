//! Keyword parsers.

use super::Input;
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
                $kw_str
                    .context(StrContext::Label(concat!("keyword '", $kw_str, "'")))
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
    pk_else => "else",
    // Types
    pk_number => "Number",
    pk_string => "String",
    pk_bool => "Bool",
    pk_color => "Color",
    pk_graphic => "Graphic",
    pk_circle => "Circle",
    pk_rect => "Rect",
    pk_text => "Text",
    // Bool literals
    pk_true => "true",
    pk_false => "false",
    // Standard functions
    pk_rgb => "rgb",
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("let"));
        assert!(is_keyword("export"));
        assert!(is_keyword("Color"));
        assert!(is_keyword("Bool"));
        assert!(is_keyword("true"));
        assert!(is_keyword("false"));
        assert!(!is_keyword("not_a_keyword"));
    }

    #[test]
    fn test_keyword_parsers() {
        let mut input = Input::new("let  ");
        assert_eq!(pk_let.parse_next(&mut input).unwrap(), "let");
        assert_eq!(*input, "  ");

        let mut input = Input::new("Number\n");
        assert_eq!(pk_number.parse_next(&mut input).unwrap(), "Number");
        assert_eq!(*input, "\n");

        let mut input = Input::new("not_a_keyword");
        assert!(pk_let.parse_next(&mut input).is_err());
    }
}
