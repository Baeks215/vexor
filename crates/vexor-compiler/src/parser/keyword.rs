//! Keyword parsers.

use super::Input;
use crate::ir::ast::Const;
use winnow::error::StrContext;
use winnow::{ModalResult, Parser};

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
            $kw_str.value($variant)
                .context(StrContext::Label(concat!("keyword '", $kw_str, "'")))
                .parse_next(input)
        }
    };

    // Without type variant — returns &'a str
    (@fn $func_name:ident, $kw_str:expr) => {
        #[doc = concat!("Parses the `", $kw_str, "` keyword.")]
        pub fn $func_name<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
            $kw_str
                .context(StrContext::Label(concat!("keyword '", $kw_str, "'")))
                .parse_next(input)
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
    // Graphic Literals
    pk_circle => "Circle"; Graphic : Graphic::Circle,
    pk_rect => "Rect"; Graphic : Graphic::Rect,
    pk_text => "Text"; Graphic : Graphic::Text,
    // Bool literals
    pk_true => "true",
    pk_false => "false",
    // List literals
    pk_nil => "Nil",
    // Color Literal
    pk_rgb => "rgb",
    // Standard functions
    pk_rad => "rad"; Std : Std::Rad,
    pk_sin => "sin"; Std : Std::Sin,
    pk_cos => "cos"; Std : Std::Cos,
    pk_tan => "tan"; Std : Std::Tan,
    pk_map => "map"; Std : Std::Map,
    // Constants
    pk_pi => "pi"; Const: Const::Pi
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Graphic {
    Circle,
    Rect,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Std {
    Rad,
    Sin,
    Cos,
    Tan,
    Map,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("let"));
        assert!(is_keyword("export"));
        assert!(is_keyword("true"));
        assert!(is_keyword("false"));
        assert!(!is_keyword("not_a_keyword"));
    }

    #[test]
    fn test_keyword_parsers() {
        let mut input = Input::new("let  ");
        assert_eq!(pk_let.parse_next(&mut input).unwrap(), "let");
        assert_eq!(*input, "  ");

        let mut input = Input::new("export\n");
        assert_eq!(pk_export.parse_next(&mut input).unwrap(), "export");
        assert_eq!(*input, "\n");

        let mut input = Input::new("not_a_keyword");
        assert!(pk_let.parse_next(&mut input).is_err());
    }
}
