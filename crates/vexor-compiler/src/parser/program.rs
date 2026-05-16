//! Parser: Text -> AST

use winnow::ascii::dec_uint;
use winnow::combinator::{alt, cut_err, eof, fail, opt, preceded, separated, separated_pair};
use winnow::error::{ContextError, ParseError};
use winnow::{ModalResult, Parser, Result};

use crate::ir::ast::{self, ProgramUnit};
use crate::parser::expr::p_expr;
use crate::parser::function::p_function_def;
use crate::parser::keyword::p_user_ident;
use crate::parser::{Input, ParserExt, exp_char, exp_string, expected, newline1, p_mws};
use crate::parser::{delim, keyword as k};

/// Parses variable assignment `x = expr`
pub fn p_assignment_raw<'a>(input: &mut Input<'a>) -> ModalResult<(String, ast::Expr)> {
    (
        p_user_ident.ws(),
        preceded(exp_string("=").mws(), cut_err(p_expr)),
    )
        .parse_next(input)
}

/// Parses variable assignment with `let x = expr`
fn p_assignment<'a>(input: &mut Input<'a>) -> ModalResult<(String, ast::Expr)> {
    preceded(k::pk_val.ws(), cut_err(p_assignment_raw)).parse_next(input)
}

/// Parses an export `export expr`
fn p_export<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    preceded(k::pk_export.ws(), p_expr.label("graphic expression")).parse_next(input)
}

fn p_setting<'a>(input: &mut Input<'a>) -> ModalResult<ast::Setting> {
    preceded(
        k::pk_set.ws(),
        alt((
            preceded(
                "canvas",
                cut_err(delim(
                    '(',
                    separated_pair(
                        dec_uint.expected("unsigned integer"),
                        exp_char(',').ws(),
                        dec_uint.expected("unsigned integer"),
                    ),
                    ')',
                )),
            )
            .map(|(width, height)| ast::Setting::Canvas { width, height })
            .ws(),
            fail.expected_lit("canvas"),
        )),
    )
    .parse_next(input)
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
            p_setting.map(ProgramUnit::Setting),
            fail.expected_lit("fn")
                .expected_lit("let")
                .expected_lit("export")
                .expected_lit("set"),
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
