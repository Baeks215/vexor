//! Parser: Text -> AST

use crate::ir::ast;
use crate::parser::expr::p_expr;
use crate::parser::keyword::{pk_export, pk_fn, pk_let, pk_where};
use crate::parser::{
    Input, ParserExt, braced, bracketed, comma_list, newline1, p_identifier, p_mws,
};
use itertools::{Either, Itertools};
use winnow::combinator::{alt, cut_err, delimited, opt, preceded, separated, terminated};
use winnow::error::{ContextError, ParseError};
use winnow::{ModalResult, Parser, Result};

#[derive(Debug, Clone)]
enum ProgramUnit {
    Assignment(ast::Assignment),
    Function(ast::Function),
}

/// Parses variable assignment `x = expr`
fn p_assignment_raw<'a>(input: &mut Input<'a>) -> ModalResult<ast::Assignment> {
    (p_identifier.ws(), preceded("=".mws(), cut_err(p_expr)))
        .map(|(i, e)| ast::Assignment {
            identifier: i.to_string(),
            value: e,
        })
        .parse_next(input)
}

/// Parses variable assignment with `let x = expr`
fn p_assignment<'a>(input: &mut Input<'a>) -> ModalResult<ast::Assignment> {
    preceded(pk_let.ws(), cut_err(p_assignment_raw)).parse_next(input)
}

fn p_function<'a>(input: &mut Input<'a>) -> ModalResult<ast::Function> {
    (preceded(
        pk_fn.ws(),
        cut_err((
            p_identifier,                                  // function name
            bracketed(comma_list(0.., p_identifier)).ws(), // parameters
            preceded("=".mws(), p_expr),                   // return expression
            opt(preceded(
                (p_mws, pk_where.ws()),
                cut_err(braced(separated(0.., p_assignment_raw, newline1))),
            )), // where scope
        )),
    ))
    .ws()
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
        p_mws,
        (
            opt(terminated(
                separated(0.., p_program_unit, newline1),
                newline1,
            ))
            .map(|u| u.unwrap_or_default()),
            separated(1.., p_export, newline1),
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
        p_mws,
    )
    .parse(input)
}
