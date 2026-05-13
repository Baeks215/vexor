//! Parser: Text -> AST

use winnow::combinator::{alt, cut_err, eof, fail, opt, preceded, separated};
use winnow::error::{ContextError, ParseError};
use winnow::{ModalResult, Parser, Result};

use crate::ir::ast::{self, ProgramUnit};
use crate::parser::expr::p_expr;
use crate::parser::keyword::{pk_export, pk_fn, pk_let, pk_where};
use crate::parser::{
    Input, ParserExt, braced, bracketed, comma_list, exp_string, expected, newline1, p_identifier,
    p_mws,
};

/// Parses variable assignment `x = expr`
fn p_assignment_raw<'a>(input: &mut Input<'a>) -> ModalResult<(String, ast::Expr)> {
    (
        p_identifier.map(str::to_string).ws(),
        preceded(exp_string("=").mws(), cut_err(p_expr)),
    )
        .parse_next(input)
}

/// Parses variable assignment with `let x = expr`
fn p_assignment<'a>(input: &mut Input<'a>) -> ModalResult<(String, ast::Expr)> {
    preceded(pk_let.ws(), cut_err(p_assignment_raw)).parse_next(input)
}

/// Parses a function definition `fn name(params) = expr`
///   Optional where clause `where { x = a \n ... }`
fn p_function_def<'a>(input: &mut Input<'a>) -> ModalResult<(String, ast::Function)> {
    (preceded(
        pk_fn.ws(),
        cut_err((
            p_identifier,                                  // function name
            bracketed(comma_list(0.., p_identifier)).ws(), // parameters
            preceded(exp_string("=").mws(), p_expr),       // return expression
            opt(preceded(
                (p_mws, pk_where.ws()),
                cut_err(braced(separated(0.., p_assignment_raw, newline1))),
            )), // where scope
        )),
    ))
    .ws()
    .map(|(name, params, return_expr, scope): (_, Vec<&str>, _, _)| {
        (
            name.to_string(),
            ast::Function {
                params: params.into_iter().map(str::to_string).collect(),
                scope: scope.unwrap_or_default(),
                return_expr: Box::new(return_expr),
            },
        )
    })
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
        1..,
        alt((
            p_function_def.map(|(identifier, func)| ProgramUnit::Function { identifier, func }),
            p_assignment.map(|(identifier, value)| ProgramUnit::Assignment { identifier, value }),
            p_export.map(|e| {
                export_count += 1;
                ProgramUnit::Export(e)
            }),
            fail.expected_lit("fn")
                .expected_lit("let")
                .expected_lit("export"),
        )),
        newline1,
    )
    .parse_next(input)?;

    // Discard rest of whitespace and expect eof
    let newlines = opt(newline1).parse_next(input)?;
    match newlines {
        Some(_) => {
            // If separated by newlines, expect more program units
            eof.expected_lit("fn")
                .expected_lit("let")
                .expected_lit("export")
                .parse_next(input)?;
        }
        None => {
            // Otherwise invalid character
            eof.label("character").parse_next(input)?;
        }
    }

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
