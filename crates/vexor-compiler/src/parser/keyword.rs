//! Keyword parsers and identifier classification.

use std::rc::Rc;

use winnow::ascii::alpha1;
use winnow::combinator::{not, terminated};
use winnow::token::take_while;
use winnow::{ModalResult, Parser};

use super::{Input, ParserExt};
use crate::ir::Ident;
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

/// Classified symbol.
#[derive(Debug, Clone)]
pub enum Symbol {
    Std(Std),
    Const(Const),
    /// User identifier, interned (see [`crate::ir::Ident`]) so binding it into scopes is a bump.
    User(Ident),
}

/// Classifies keywords from user identifiers.
fn classify_kw(s: &str) -> Symbol {
    match s {
        // Trig
        "rad" => Symbol::Std(Std::Rad),
        "deg" => Symbol::Std(Std::Deg),
        "sin" => Symbol::Std(Std::Sin),
        "cos" => Symbol::Std(Std::Cos),
        "tan" => Symbol::Std(Std::Tan),
        // Math
        "sinh" => Symbol::Std(Std::Sinh),
        "cosh" => Symbol::Std(Std::Cosh),
        "tanh" => Symbol::Std(Std::Tanh),
        "asinh" => Symbol::Std(Std::Asinh),
        "acosh" => Symbol::Std(Std::Acosh),
        "atanh" => Symbol::Std(Std::Atanh),
        "asin" => Symbol::Std(Std::Asin),
        "acos" => Symbol::Std(Std::Acos),
        "atan" => Symbol::Std(Std::Atan),
        "atan2" => Symbol::Std(Std::Atan2),
        "round" => Symbol::Std(Std::Round),
        "floor" => Symbol::Std(Std::Floor),
        "ceil" => Symbol::Std(Std::Ceil),
        "abs" => Symbol::Std(Std::Abs),
        "log" => Symbol::Std(Std::Log),
        "exp" => Symbol::Std(Std::Exp),
        "max" => Symbol::Std(Std::Max),
        "min" => Symbol::Std(Std::Min),
        "clamp" => Symbol::Std(Std::Clamp),
        "magnitude" => Symbol::Std(Std::Magnitude),
        "normalize" => Symbol::Std(Std::Normalize),
        "dot" => Symbol::Std(Std::Dot),
        // List
        "map" => Symbol::Std(Std::Map),
        "filter" => Symbol::Std(Std::Filter),
        "drop" => Symbol::Std(Std::Drop),
        "take" => Symbol::Std(Std::Take),
        "dropWhile" => Symbol::Std(Std::DropWhile),
        "takeWhile" => Symbol::Std(Std::TakeWhile),
        "foldl" => Symbol::Std(Std::Foldl),
        "foldr" => Symbol::Std(Std::Foldr),
        "zip" => Symbol::Std(Std::Zip),
        "zipWith" => Symbol::Std(Std::ZipWith),
        "flatMap" => Symbol::Std(Std::FlatMap),
        "enumerate" => Symbol::Std(Std::Enumerate),
        "len" => Symbol::Std(Std::Len),
        "reverse" => Symbol::Std(Std::Reverse),
        "find" => Symbol::Std(Std::Find),
        "sort" => Symbol::Std(Std::Sort),
        "sortBy" => Symbol::Std(Std::SortBy),
        "repeat" => Symbol::Std(Std::Repeat),
        "nth" => Symbol::Std(Std::Nth),
        "head" => Symbol::Std(Std::Head),
        "tail" => Symbol::Std(Std::Tail),
        "last" => Symbol::Std(Std::Last),
        "init" => Symbol::Std(Std::Init),
        "isEmpty" => Symbol::Std(Std::IsEmpty),
        "sum" => Symbol::Std(Std::Sum),
        "product" => Symbol::Std(Std::Product),
        "concat" => Symbol::Std(Std::Concat),
        // Tuple
        "fst" => Symbol::Std(Std::Fst),
        "snd" => Symbol::Std(Std::Snd),
        // Color constructors
        "rgb" => Symbol::Std(Std::Rgb),
        "rgba" => Symbol::Std(Std::Rgba),
        "hsl" => Symbol::Std(Std::Hsl),
        "hsla" => Symbol::Std(Std::Hsla),
        // Graphic constructors
        "Circle" => Symbol::Std(Std::Circle),
        "Ellipse" => Symbol::Std(Std::Ellipse),
        "Rect" => Symbol::Std(Std::Rect),
        "Text" => Symbol::Std(Std::Text),
        "Group" => Symbol::Std(Std::Group),
        "Line" => Symbol::Std(Std::Line),
        "Curve" => Symbol::Std(Std::Curve),
        "Path" => Symbol::Std(Std::Path),
        "sample" => Symbol::Std(Std::Sample),
        // Graphic transforms
        "close" => Symbol::Std(Std::Close),
        "jumpTo" => Symbol::Std(Std::JumpTo),
        "lineTo" => Symbol::Std(Std::LineTo),
        "curveTo" => Symbol::Std(Std::CurveTo),
        "move" => Symbol::Std(Std::Move),
        "scale" => Symbol::Std(Std::Scale),
        "rotate" => Symbol::Std(Std::Rotate),
        "mirrorX" => Symbol::Std(Std::MirrorX),
        "mirrorY" => Symbol::Std(Std::MirrorY),
        "fill" => Symbol::Std(Std::Fill),
        "strokeWidth" => Symbol::Std(Std::StrokeWidth),
        "strokeColor" => Symbol::Std(Std::StrokeColor),
        "strokeJoin" => Symbol::Std(Std::StrokeJoin),
        "strokeCap" => Symbol::Std(Std::StrokeCap),
        "opacity" => Symbol::Std(Std::Opacity),
        "setId" => Symbol::Std(Std::SetId),
        // Constants
        "PI" => Symbol::Const(Const::Pi),
        // User
        other => Symbol::User(Rc::from(other)),
    }
}

/// Parses characters and classifies as an Std/Const/User identifier.
pub fn p_ident<'a>(input: &mut Input<'a>) -> ModalResult<Symbol> {
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
pub fn p_user_ident<'a>(input: &mut Input<'a>) -> ModalResult<Ident> {
    p_ident
        .verify_map(|id| match id {
            Symbol::User(s) => Some(s),
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
            Symbol::User(s) => assert_eq!(&*s, "foo_bar_123"),
            other => panic!("expected User, got {:?}", other),
        }

        let mut input = Input::new("map");
        match p_ident.parse_next(&mut input).unwrap() {
            Symbol::Std(Std::Map) => (),
            other => panic!("expected Std::Map, got {:?}", other),
        }

        let mut input = Input::new("PI");
        match p_ident.parse_next(&mut input).unwrap() {
            Symbol::Const(Const::Pi) => (),
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
        assert_eq!(&*p_user_ident.parse_next(&mut input).unwrap(), "my_var");

        // Std name rejected
        let mut input = Input::new("map");
        assert!(p_user_ident.parse_next(&mut input).is_err());

        // Const rejected
        let mut input = Input::new("PI");
        assert!(p_user_ident.parse_next(&mut input).is_err());
    }
}
