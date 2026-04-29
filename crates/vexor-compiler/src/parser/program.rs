//! Parser: Text -> AST

use crate::ir::ast;
use crate::ir::typed::{GraphicType, Type};
use crate::parser::expr::p_expr;
use crate::parser::keyword::{
    pk_bool, pk_circle, pk_color, pk_export, pk_fn, pk_graphic, pk_let, pk_number, pk_rect,
    pk_string, pk_text, pk_where,
};
use crate::parser::{Input, WhiteSpaceParser, braced, bracketed, p_identifier};
use itertools::{Either, Itertools};
use winnow::ascii::{line_ending, multispace0};
use winnow::combinator::{alt, delimited, opt, preceded, separated, separated_pair};
use winnow::error::{ContextError, ParseError};
use winnow::{ModalResult, Parser, Result};

#[derive(Debug, Clone, PartialEq)]
enum ProgramUnit {
    Assignment(ast::Assignment),
    Function(ast::Function),
}

fn p_type<'a>(input: &mut Input<'a>) -> ModalResult<Type> {
    alt((
        pk_number.map(|_| Type::Number),
        pk_string.map(|_| Type::String),
        pk_bool.map(|_| Type::Bool),
        pk_color.map(|_| Type::Color),
        pk_graphic.map(|_| Type::Graphic),
        pk_circle.map(|_| Type::GType(GraphicType::Circle)),
        pk_rect.map(|_| Type::GType(GraphicType::Rect)),
        pk_text.map(|_| Type::GType(GraphicType::Text)),
    ))
    .ws()
    .parse_next(input)
}

fn p_assignment<'a>(input: &mut Input<'a>) -> ModalResult<ast::Assignment> {
    (
        pk_let.ws(),
        p_identifier,
        ":".ws(),
        p_type,
        "=".ws(),
        p_expr,
    )
        .map(|(_, i, _, t, _, e)| ast::Assignment {
            ty: t,
            identifier: i.to_string(),
            value: e,
        })
        .parse_next(input)
}

fn p_function<'a>(input: &mut Input<'a>) -> ModalResult<ast::Function> {
    (
        preceded(pk_fn.ws(), p_identifier), // function name
        bracketed(separated(
            0..,
            separated_pair(p_identifier, ":".ws(), p_type),
            ",".ws(),
        )), // parameters
        preceded(":".ws(), p_type),         // return type
        preceded("=".mws(), p_expr),        // return expression
        opt(preceded(
            pk_where.ws(),
            braced(separated(0.., p_assignment, multispace0)).mws(),
        )), // where scope
    )
        .map(
            |(name, params, return_type, return_expr, scope): (_, Vec<(&str, Type)>, _, _, _)| {
                ast::Function {
                    name: name.to_string(),
                    params: params
                        .into_iter()
                        .map(|(n, t)| (n.to_string(), t))
                        .collect(),
                    scope: scope.unwrap_or_default(),
                    return_expr: (return_expr, return_type),
                }
            },
        )
        .parse_next(input)
}

fn p_program_unit<'a>(input: &mut Input<'a>) -> ModalResult<ProgramUnit> {
    alt((
        p_function.map(|f| ProgramUnit::Function(f)),
        p_assignment.map(|s| ProgramUnit::Assignment(s)),
    ))
    .parse_next(input)
}

fn p_export<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    preceded(pk_export.ws(), p_expr).parse_next(input)
}

/// Parses a program from the given input string.
///   Text -> AST
pub fn parse_program<'a>(
    input: &'a str,
) -> Result<ast::Program, ParseError<Input<'a>, ContextError>> {
    let input = Input::new(input);
    delimited(
        multispace0,
        separated_pair(
            separated(0.., p_program_unit, line_ending.ws()),
            multispace0,
            separated(0.., p_export, line_ending.ws()),
        )
        .map(|(units, exports): (Vec<ProgramUnit>, Vec<ast::Expr>)| {
            let (functions, statements) = units.into_iter().partition_map(|u| match u {
                ProgramUnit::Function(f) => Either::Left(f),
                ProgramUnit::Assignment(s) => Either::Right(s),
            });
            ast::Program {
                functions,
                scope: statements,
                exports,
            }
        }),
        multispace0,
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p_statement_assignment() {
        let mut input = Input::new("let x: number = 10");
        let res = p_assignment.parse_next(&mut input).unwrap();
        let ast::Assignment {
            ty,
            identifier,
            value,
        } = res;
        assert_eq!(ty, Type::Number);
        assert_eq!(identifier, "x");
        assert_eq!(value, ast::Expr::LNumber(10.0));

        let mut input = Input::new("let my_var: string = \"hello\"");
        let res = p_assignment.parse_next(&mut input).unwrap();
        let ast::Assignment {
            ty,
            identifier,
            value,
        } = res;
        assert_eq!(ty, Type::String);
        assert_eq!(identifier, "my_var");
        assert_eq!(value, ast::Expr::LString("hello".to_string()));

        let mut input = Input::new("let b: bool = true");
        let res = p_assignment.parse_next(&mut input).unwrap();
        let ast::Assignment {
            ty,
            identifier,
            value,
        } = res;
        assert_eq!(ty, Type::Bool);
        assert_eq!(identifier, "b");
        assert_eq!(value, ast::Expr::LBool(true));
    }

    #[test]
    fn test_p_export() {
        let mut input = Input::new("export Circle { radius: 10 }");
        let res = p_export.parse_next(&mut input).unwrap();
        if let ast::Expr::LObject(ast::Object { name, fields }) = res {
            assert_eq!(name, "Circle");
            assert_eq!(
                fields,
                vec![("radius".to_string(), ast::Expr::LNumber(10.0))]
            );
        } else {
            panic!("Expected Export, got {:?}", res);
        }
    }

    #[test]
    fn test_p_function() {
        let input = "fn double(x: number): number = y where {\nlet y: number = x + x\n}";
        let mut input = Input::new(input);
        let res = p_function.parse_next(&mut input).unwrap();
        assert_eq!(res.name, "double");
        assert_eq!(res.params, vec![("x".to_string(), Type::Number)]);
        assert_eq!(res.return_expr.1, Type::Number);
        assert_eq!(res.scope.len(), 1);
        let ast::Assignment { identifier, .. } = &res.scope[0];
        assert_eq!(identifier, "y");

        // Zero-param, empty-body function
        let mut input = Input::new("fn five(): number = 5");
        let res = p_function.parse_next(&mut input).unwrap();
        assert_eq!(res.name, "five");
        assert!(res.params.is_empty());
        assert!(res.scope.is_empty());
        assert_eq!(res.return_expr.0, ast::Expr::LNumber(5.0));
    }

    #[test]
    fn test_parse_program_with_function() {
        let input = "fn mk(r: number): number = r + 1\nexport Circle { radius: mk(5) }";
        let res = parse_program(input).unwrap();
        assert_eq!(res.functions.len(), 1);
        assert!(res.scope.is_empty());
        assert_eq!(res.exports.len(), 1);

        if let ast::Expr::LObject(ast::Object { name, fields }) = &res.exports[0] {
            assert_eq!(name, "Circle");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "radius");
            match &fields[0].1 {
                ast::Expr::Call { function, args } => {
                    assert_eq!(function, "mk");
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], ast::Expr::LNumber(5.0));
                }
                other => panic!("Expected Call, got {:?}", other),
            }
        } else {
            panic!("Expected Circle export, got {:?}", res.exports[0]);
        }
    }

    #[test]
    fn test_parse_program() {
        let input = "  let x: number = 10  \n \t export Circle { radius: x }  \n";
        let res = parse_program(input).unwrap();
        assert_eq!(res.scope.len(), 1);

        let ast::Assignment {
            ty,
            identifier,
            value,
        } = &res.scope[0];
        assert_eq!(ty, &Type::Number);
        assert_eq!(identifier, "x");
        assert_eq!(*value, ast::Expr::LNumber(10.0));

        if let ast::Expr::LObject(ast::Object { name, fields }) = &res.exports[0] {
            assert_eq!(name, "Circle");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "radius");
            assert_eq!(fields[0].1, ast::Expr::Variable("x".to_string()));
        } else {
            panic!("Expected Export, got {:?}", res.exports[0]);
        }
    }
}
