use winnow::combinator::{alt, cut_err, opt, preceded, repeat, separated};
use winnow::{ModalResult, Parser};

use crate::ir::ast;
use crate::parser::expr::p_expr;
use crate::parser::program::p_assignment_raw;
use crate::parser::{Input, ParserExt, comma_list, exp_string, newline1, p_identifier, p_mws};
use crate::parser::{delim, keyword as k};

/// Parses a std function.
pub fn p_std<'a>(input: &mut Input<'a>) -> ModalResult<ast::Std> {
    alt((
        // Trig functions
        alt((k::pk_rad, k::pk_sin, k::pk_cos, k::pk_tan)),
        // List
        alt((
            k::pk_map,
            k::pk_filter,
            k::pk_flat_map,
            k::pk_drop_while,
            k::pk_drop,
            k::pk_take_while,
            k::pk_take,
            k::pk_foldl,
            k::pk_foldr,
        )),
        alt((
            k::pk_zip_with,
            k::pk_zip,
            k::pk_enumerate,
            k::pk_len,
            k::pk_reverse,
            k::pk_find,
            k::pk_sort_by,
            k::pk_sort,
        )),
        // Graphic constructors
        alt((k::pk_circle, k::pk_rect, k::pk_text, k::pk_group)),
        // Graphic functions
        alt((
            k::pk_move,
            k::pk_scale,
            k::pk_rotate,
            k::pk_fill,
            k::pk_stroke,
        )),
    ))
    .ws()
    .parse_next(input)
}

/// Parses a lambda expression
pub fn p_lambda<'a>(input: &mut Input<'a>) -> ModalResult<ast::Function> {
    (
        alt((
            p_identifier.map(|id| vec![vec![id.to_string()]]),
            repeat(
                // Curried parameters
                1..,
                delim::<_, Vec<_>>('(', comma_list(0.., p_identifier.map(str::to_string)), ')'),
            ),
        ))
        .mws(),
        preceded("->".mws(), cut_err(p_expr).label("lambda body")),
    )
        .map(|(params, e)| build_curried_function(params, e, vec![]))
        .parse_next(input)
}

/// Parses a function definition `fn name(params) = expr`
///   Optional where clause `where { x = a \n ... }`
pub fn p_function_def<'a>(input: &mut Input<'a>) -> ModalResult<(String, ast::Function)> {
    (preceded(
        k::pk_fn.ws(),
        cut_err((
            p_identifier.map(str::to_string), // function name
            repeat(
                // Curried parameters
                1..,
                delim::<_, Vec<_>>('(', comma_list(0.., p_identifier.map(str::to_string)), ')'),
            )
            .ws(), // parameters
            preceded(exp_string("=").mws(), p_expr), // return expression
            opt(preceded(
                (p_mws, k::pk_where.ws()),
                cut_err(delim('{', separated(0.., p_assignment_raw, newline1), '}')),
            )), // where scope
        )),
    ))
    .ws()
    .map(|(name, curried_params, return_expr, scope)| {
        let func = build_curried_function(curried_params, return_expr, scope.unwrap_or_default());
        (name, func)
    })
    .parse_next(input)
}

/// Builds a curried function to the function ast node
fn build_curried_function(
    mut curried_params: Vec<Vec<String>>,
    return_expr: ast::Expr,
    scope: Vec<(String, ast::Expr)>,
) -> ast::Function {
    let last_params = curried_params.pop().unwrap(); // repeat(1..) guarantees Some
    // Last curried function is the main function
    let mut acc_function = ast::Function {
        params: last_params,
        scope,
        return_expr: Box::new(return_expr),
    };
    // Curry remaining parameters in reverse
    for params in curried_params.into_iter().rev() {
        acc_function = ast::Function {
            params,
            scope: vec![],
            return_expr: Box::new(ast::Expr::Function(acc_function)),
        };
    }
    acc_function
}
