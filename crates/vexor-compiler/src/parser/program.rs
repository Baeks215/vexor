//! Parser: Text -> AST

use crate::ir::ast;
use crate::parser::expr::p_expr;
use crate::parser::keyword::{pk_export, pk_fn, pk_let, pk_where};
use crate::parser::{Input, WhiteSpaceParser, braced, bracketed, p_identifier};
use itertools::{Either, Itertools};
use winnow::ascii::{multispace0, multispace1};
use winnow::combinator::{alt, delimited, opt, preceded, separated, terminated};
use winnow::error::{ContextError, ParseError};
use winnow::{ModalResult, Parser, Result};

#[derive(Debug, Clone)]
enum ProgramUnit {
    Assignment(ast::Assignment),
    Function(ast::Function),
}

fn p_assignment<'a>(input: &mut Input<'a>) -> ModalResult<ast::Assignment> {
    (
        preceded(pk_let.ws(), p_identifier.ws()),
        preceded("=".ws(), p_expr),
    )
        .map(|(i, e)| ast::Assignment {
            identifier: i.to_string(),
            value: e,
        })
        .parse_next(input)
}

fn p_function<'a>(input: &mut Input<'a>) -> ModalResult<ast::Function> {
    (
        preceded(pk_fn.ws(), p_identifier), // function name
        bracketed(separated(0.., p_identifier.ws(), ",".ws())) // parameters
            .ws(),
        preceded("=".mws(), p_expr), // return expression
        opt(preceded(
            pk_where.ws(),
            braced(separated(0.., p_assignment, multispace0)).ws(),
        )), // where scope
    )
        .map(
            |(name, params, return_expr, scope): (_, Vec<&str>, _, _)| ast::Function {
                name: name.to_string(),
                params: params.into_iter().map(str::to_string).collect(),
                scope: scope.unwrap_or_default(),
                return_expr: return_expr,
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
        (
            opt(terminated(
                separated(0.., p_program_unit, multispace1),
                multispace1,
            ))
            .map(|u| u.unwrap_or_default()),
            separated(0.., p_export, multispace1),
        )
            .map(|(units, exports): (Vec<_>, Vec<_>)| {
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
