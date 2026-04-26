//! Parser for graphic components

use crate::ir::ast;
use crate::parser::expr::p_expr;
use crate::parser::{Input, WhiteSpaceParser, braced, p_identifier};
use winnow::combinator::{separated, separated_pair};
use winnow::{ModalResult, Parser};

pub fn p_object<'a>(input: &mut Input<'a>) -> ModalResult<ast::Object> {
    (
        p_identifier.map(str::to_string), // Object keywords
        braced(separated(
            0..,
            separated_pair(p_identifier.map(str::to_string), ':'.ws(), p_expr),
            ','.mws(),
        )),
    )
        .ws()
        .map(|(name, fields)| ast::Object { name, fields })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p_object_circle() {
        let mut input = Input::new("Circle { radius: 10 }");
        let res = p_object.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Object {
                name: "Circle".to_string(),
                fields: vec![("radius".to_string(), ast::Expr::LNumber(10.0))],
            }
        );
        assert_eq!(*input, "");

        let mut input = Input::new("Circle { radius: 5 + 5 }");
        let res = p_object.parse_next(&mut input).unwrap();
        assert_eq!(
            res.fields[0].1,
            ast::Expr::Binary {
                operator: ast::OpBin::Add,
                left: Box::new(ast::Expr::LNumber(5.0)),
                right: Box::new(ast::Expr::LNumber(5.0)),
            }
        );
    }

    #[test]
    fn test_p_object_rect() {
        let mut input = Input::new("Rect { width: 10, height: 20 }");
        let res = p_object.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Object {
                name: "Rect".to_string(),
                fields: vec![
                    ("width".to_string(), ast::Expr::LNumber(10.0)),
                    ("height".to_string(), ast::Expr::LNumber(20.0)),
                ],
            }
        );
        assert_eq!(*input, "");
    }

    #[test]
    fn test_p_object_text() {
        let mut input = Input::new("Text { content: \"hello\" }");
        let res = p_object.parse_next(&mut input).unwrap();
        assert_eq!(
            res,
            ast::Object {
                name: "Text".to_string(),
                fields: vec![(
                    "content".to_string(),
                    ast::Expr::LString("hello".to_string()),
                )],
            }
        );
        assert_eq!(*input, "");
    }
}
