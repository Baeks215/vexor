//! Parser: Text -> AST

use winnow::combinator::{alt, cut_err, eof, opt, preceded, separated};
use winnow::error::{ContextError, ParseError};
use winnow::{ModalResult, Parser, Result};

use crate::ir::ast::{self, ProgramUnit};
use crate::parser::expr::p_expr;
use crate::parser::keyword::{pk_export, pk_fn, pk_let, pk_where};
use crate::parser::{
    Input, ParserExt, braced, bracketed, comma_list, expected, newline1, p_identifier, p_mws,
};

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

/// Parses a function definition `fn name(params) = expr`
///   Optional where clause `where { x = a \n ... }`
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

/// Parses an export `export expr`
fn p_export<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    preceded(pk_export.ws(), p_expr.label("graphic expression")).parse_next(input)
}

/// Parses a program `fn ... let x = a ... export ...`
///   Requires at least one export
fn p_program<'a>(input: &mut Input<'a>) -> ModalResult<ast::Program> {
    p_mws.parse_next(input)?; // Discard leading whitespace
    let mut export_count = 0;
    let units: Vec<_> = separated(
        0..,
        alt((
            p_function.map(|f| ProgramUnit::Function(f)),
            p_assignment.map(|s| ProgramUnit::Assignment(s)),
            p_export.map(|e| {
                export_count += 1;
                ProgramUnit::Export(e)
            }),
        )),
        newline1,
    )
    .mws() // Discard trailing whitespace
    .parse_next(input)?;

    eof.expected_lit("fn")
        .expected_lit("let")
        .expected_lit("export")
        .parse_next(input)?;

    if export_count == 0 {
        return Err(expected("at least one export", input));
    }

    Ok(ast::Program { units })
}

/// Parses a program from the given input string.
///   Text -> AST
pub fn parse_program<'a>(
    input: &'a str,
) -> Result<ast::Program, ParseError<Input<'a>, ContextError>> {
    let input = Input::new(input);
    p_program.parse(input)
}
