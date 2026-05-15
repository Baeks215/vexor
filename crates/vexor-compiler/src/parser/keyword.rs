//! Keyword parsers.

use winnow::ascii::alpha1;
use winnow::combinator::{not, terminated};
use winnow::{ModalResult, Parser};

use super::{Input, ParserExt};
use crate::ir::ast::{Const, Std};

/// Macro to define a set of keyword parsers
macro_rules! define_keywords {
    (
        // Comma-separated list of `parser_name => "keyword"` or `parser_name => "keyword" ; return type : variant`
        $($func_name:ident => $kw_str:expr $(; $type:ty : $variant:expr)?),* $(,)?
    ) => {
        // Generates a parser function for every keyword in the list
        $(
            define_keywords!(@fn $func_name, $kw_str $(, $type, $variant)?);
        )*

        /// Checks if a string is a keyword.
        pub fn is_keyword(s: &str) -> bool {
            matches!(s, $($kw_str)|*)
        }
    };

    // With type variant — returns the specified type
    (@fn $func_name:ident, $kw_str:expr, $type:ty, $variant:expr) => {
        #[doc = concat!("Parses the `", $kw_str, "` keyword.")]
        pub fn $func_name<'a>(input: &mut Input<'a>) -> ModalResult<$type> {
            terminated($kw_str, not(alpha1)).value($variant)
                .expected_lit($kw_str)
                .parse_next(input)
        }
    };

    // Without type variant — returns &'a str
    (@fn $func_name:ident, $kw_str:expr) => {
        #[doc = concat!("Parses the `", $kw_str, "` keyword.")]
        pub fn $func_name<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
            terminated($kw_str, not(alpha1))
                .expected_lit($kw_str)
                .parse_next(input)
        }
    };
}

define_keywords! {
    // Defined keywords
    pk_set => "set",
    pk_val => "val",
    pk_export => "export",
    pk_fn => "fn",
    pk_where => "where",
    pk_match => "match",
    pk_if => "if",
    pk_else => "else",
    // Bool literals
    pk_true => "true",
    pk_false => "false",
    // List literals
    pk_nil => "Nil",
    // Color Literal
    pk_rgb => "rgb",
    // Standard functions
    pk_rad => "rad"; Std: Std::Rad,
    pk_sin => "sin"; Std: Std::Sin,
    pk_cos => "cos"; Std: Std::Cos,
    pk_tan => "tan"; Std: Std::Tan,
    pk_map => "map"; Std: Std::Map,
    pk_circle => "Circle"; Std: Std::Circle,
    pk_rect => "Rect"; Std: Std::Rect,
    pk_text => "Text"; Std: Std::Text,
    pk_group => "Group"; Std: Std::Group,
    pk_move => "move"; Std: Std::Move,
    pk_scale => "scale"; Std: Std::Scale,
    pk_rotate => "rotate"; Std: Std::Rotate,
    pk_fill => "fill"; Std: Std::Fill,
    pk_stroke => "stroke"; Std: Std::Stroke,
    // Constants
    pk_pi => "pi"; Const: Const::Pi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("val"));
        assert!(is_keyword("export"));
        assert!(is_keyword("true"));
        assert!(is_keyword("false"));
        assert!(!is_keyword("not_a_keyword"));
    }

    #[test]
    fn test_keyword_parsers() {
        let mut input = Input::new("val  ");
        assert_eq!(pk_val.parse_next(&mut input).unwrap(), "val");
        assert_eq!(*input, "  ");

        let mut input = Input::new("export\n");
        assert_eq!(pk_export.parse_next(&mut input).unwrap(), "export");
        assert_eq!(*input, "\n");

        let mut input = Input::new("not_a_keyword");
        assert!(pk_val.parse_next(&mut input).is_err());
    }
}
