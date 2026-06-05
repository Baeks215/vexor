//! Keyword parsers and identifier classification.

use winnow::ascii::alpha1;
use winnow::combinator::{not, terminated};
use winnow::token::take_while;
use winnow::{ModalResult, Parser};

use super::{Input, ParserExt};
use crate::ir::ast::{Const, Std};

/// Macro to define a set of keyword parsers
macro_rules! define_keywords {
    (
        $($func_name:ident => $kw_str:expr),* $(,)?
    ) => {
        $(
            #[doc = concat!("Parses the `", $kw_str, "` keyword.")]
            pub fn $func_name<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
                terminated($kw_str, not(alpha1))
                    .expected_lit($kw_str)
                    .parse_next(input)
            }
        )*

        /// Checks if a string is a reserved syntactic keyword.
        ///   These can never be used as user identifiers.
        pub fn is_keyword(s: &str) -> bool {
            matches!(s, $($kw_str)|*)
        }
    };
}

// Defined keywords which require special parsers
define_keywords! {
    // Syntactic keywords
    pk_set => "set",
    pk_val => "val",
    pk_export => "export",
    pk_each => "each",
    pk_fn => "fn",
    pk_where => "where",
    pk_match => "match",
    pk_if => "if",
    pk_else => "else",
    // Bool literals
    pk_true => "true",
    pk_false => "false",
    // List nil literal
    pk_nil => "Nil",
}

/// Classified identifier.
#[derive(Debug, Clone)]
pub enum Ident {
    Std(Std),
    Const(Const),
    User(String),
}

/// Classifies keywords from user identifiers.
fn classify_kw(s: &str) -> Ident {
    match s {
        // Trig
        "rad" => Ident::Std(Std::Rad),
        "deg" => Ident::Std(Std::Deg),
        "sin" => Ident::Std(Std::Sin),
        "cos" => Ident::Std(Std::Cos),
        "tan" => Ident::Std(Std::Tan),
        // Math
        "sinh" => Ident::Std(Std::Sinh),
        "cosh" => Ident::Std(Std::Cosh),
        "tanh" => Ident::Std(Std::Tanh),
        "asinh" => Ident::Std(Std::Asinh),
        "acosh" => Ident::Std(Std::Acosh),
        "atanh" => Ident::Std(Std::Atanh),
        "asin" => Ident::Std(Std::Asin),
        "acos" => Ident::Std(Std::Acos),
        "atan" => Ident::Std(Std::Atan),
        "atan2" => Ident::Std(Std::Atan2),
        "round" => Ident::Std(Std::Round),
        "floor" => Ident::Std(Std::Floor),
        "ceil" => Ident::Std(Std::Ceil),
        "abs" => Ident::Std(Std::Abs),
        "log" => Ident::Std(Std::Log),
        "exp" => Ident::Std(Std::Exp),
        "max" => Ident::Std(Std::Max),
        "min" => Ident::Std(Std::Min),
        "clamp" => Ident::Std(Std::Clamp),
        "magnitude" => Ident::Std(Std::Magnitude),
        "normalize" => Ident::Std(Std::Normalize),
        "dot" => Ident::Std(Std::Dot),
        // List
        "map" => Ident::Std(Std::Map),
        "filter" => Ident::Std(Std::Filter),
        "drop" => Ident::Std(Std::Drop),
        "take" => Ident::Std(Std::Take),
        "dropWhile" => Ident::Std(Std::DropWhile),
        "takeWhile" => Ident::Std(Std::TakeWhile),
        "foldl" => Ident::Std(Std::Foldl),
        "foldr" => Ident::Std(Std::Foldr),
        "zip" => Ident::Std(Std::Zip),
        "zipWith" => Ident::Std(Std::ZipWith),
        "flatMap" => Ident::Std(Std::FlatMap),
        "enumerate" => Ident::Std(Std::Enumerate),
        "len" => Ident::Std(Std::Len),
        "reverse" => Ident::Std(Std::Reverse),
        "find" => Ident::Std(Std::Find),
        "sort" => Ident::Std(Std::Sort),
        "sortBy" => Ident::Std(Std::SortBy),
        "repeat" => Ident::Std(Std::Repeat),
        "nth" => Ident::Std(Std::Nth),
        "head" => Ident::Std(Std::Head),
        "tail" => Ident::Std(Std::Tail),
        "last" => Ident::Std(Std::Last),
        "init" => Ident::Std(Std::Init),
        "isEmpty" => Ident::Std(Std::IsEmpty),
        "sum" => Ident::Std(Std::Sum),
        "product" => Ident::Std(Std::Product),
        "concat" => Ident::Std(Std::Concat),
        // Tuple
        "fst" => Ident::Std(Std::Fst),
        "snd" => Ident::Std(Std::Snd),
        // Color constructors
        "rgb" => Ident::Std(Std::Rgb),
        "rgba" => Ident::Std(Std::Rgba),
        "hsl" => Ident::Std(Std::Hsl),
        "hsla" => Ident::Std(Std::Hsla),
        // Graphic constructors
        "Circle" => Ident::Std(Std::Circle),
        "Ellipse" => Ident::Std(Std::Ellipse),
        "Rect" => Ident::Std(Std::Rect),
        "Text" => Ident::Std(Std::Text),
        "Group" => Ident::Std(Std::Group),
        "Line" => Ident::Std(Std::Line),
        "Curve" => Ident::Std(Std::Curve),
        "Path" => Ident::Std(Std::Path),
        "sample" => Ident::Std(Std::Sample),
        // Graphic transforms
        "close" => Ident::Std(Std::Close),
        "jumpTo" => Ident::Std(Std::JumpTo),
        "lineTo" => Ident::Std(Std::LineTo),
        "curveTo" => Ident::Std(Std::CurveTo),
        "move" => Ident::Std(Std::Move),
        "scale" => Ident::Std(Std::Scale),
        "rotate" => Ident::Std(Std::Rotate),
        "mirrorX" => Ident::Std(Std::MirrorX),
        "mirrorY" => Ident::Std(Std::MirrorY),
        "fill" => Ident::Std(Std::Fill),
        "strokeWidth" => Ident::Std(Std::StrokeWidth),
        "strokeColor" => Ident::Std(Std::StrokeColor),
        "strokeJoin" => Ident::Std(Std::StrokeJoin),
        "strokeCap" => Ident::Std(Std::StrokeCap),
        "opacity" => Ident::Std(Std::Opacity),
        "setId" => Ident::Std(Std::SetId),
        // Constants
        "PI" => Ident::Const(Const::Pi),
        // User
        other => Ident::User(other.to_string()),
    }
}

/// Parses characters and classifies as an Std/Const/User identifier.
pub fn p_ident<'a>(input: &mut Input<'a>) -> ModalResult<Ident> {
    (
        take_while(1, |c: char| c.is_alphabetic() || c == '_'),
        take_while(0.., |c: char| c.is_alphanumeric() || c == '_'),
    )
        .take()
        .label("identifier")
        // Reject keywords
        .verify(|ident: &str| !is_keyword(ident))
        .map(classify_kw)
        .parse_next(input)
}

/// Parses an identifier that must be a user binding name.
///   Rejects all keywords
pub fn p_user_ident<'a>(input: &mut Input<'a>) -> ModalResult<String> {
    p_ident
        .verify_map(|id| match id {
            Ident::User(s) => Some(s),
            _ => None,
        })
        .expected("non-keyword identifier")
        .parse_next(input)
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
        assert!(!is_keyword("map"));
        assert!(!is_keyword("PI"));
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

    #[test]
    fn test_p_ident() {
        let mut input = Input::new("foo_bar_123");
        match p_ident.parse_next(&mut input).unwrap() {
            Ident::User(s) => assert_eq!(s, "foo_bar_123"),
            other => panic!("expected User, got {:?}", other),
        }

        let mut input = Input::new("map");
        match p_ident.parse_next(&mut input).unwrap() {
            Ident::Std(Std::Map) => (),
            other => panic!("expected Std::Map, got {:?}", other),
        }

        let mut input = Input::new("PI");
        match p_ident.parse_next(&mut input).unwrap() {
            Ident::Const(Const::Pi) => (),
            other => panic!("expected Const::Pi, got {:?}", other),
        }

        // Syntactic keyword rejected
        let mut input = Input::new("val");
        assert!(p_ident.parse_next(&mut input).is_err());

        // Digit-leading rejected
        let mut input = Input::new("123");
        assert!(p_ident.parse_next(&mut input).is_err());
    }

    #[test]
    fn test_p_user_ident() {
        let mut input = Input::new("my_var");
        assert_eq!(p_user_ident.parse_next(&mut input).unwrap(), "my_var");

        // Std name rejected
        let mut input = Input::new("map");
        assert!(p_user_ident.parse_next(&mut input).is_err());

        // Const rejected
        let mut input = Input::new("PI");
        assert!(p_user_ident.parse_next(&mut input).is_err());
    }
}
