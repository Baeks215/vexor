use std::rc::Rc;

use winnow::combinator::{alt, cut_err, opt, preceded, repeat, separated};
use winnow::stream::Stream;
use winnow::{ModalResult, Parser};

use crate::ir::Ident;
use crate::ir::ast::{self, SpanExpr, Spanned};
use crate::parser::error::CtxErrBuilder;
use crate::parser::expr::p_expr;
use crate::parser::keyword::p_user_ident;
use crate::parser::program::p_assignment_raw;
use crate::parser::{Input, ParserExt, comma_list, exp_string, newline1, p_mws};
use crate::parser::{delim, keyword as k};

/// Parses a lambda expression
pub fn p_lambda<'a>(input: &mut Input<'a>) -> ModalResult<ast::Function> {
    let params: Vec<Vec<Ident>> = alt((
        p_user_ident.map(|id| vec![vec![id]]),
        repeat(
            // Curried parameters
            1..,
            delim::<_, Vec<_>>('(', comma_list(0.., p_user_ident), ')'),
        ),
    ))
    .mws()
    .parse_next(input)?;
    "->".parse_next(input)?;

    let start = input.checkpoint();

    p_mws.parse_next(input)?;
    let body = p_expr.parse_next(input).map_err(|_| {
        CtxErrBuilder::from_checkpoint(input, &start)
            .label("lambda body")
            .expected("return expression")
            .err
    })?;

    Ok(build_curried_function(params, body, vec![]))
}

/// Parses a function definition `fn name(params) = expr`
///   Optional where clause `where { x = a \n ... }`
pub fn p_function_def<'a>(input: &mut Input<'a>) -> ModalResult<(Ident, ast::Function)> {
    (preceded(
        k::pk_fn.ws(),
        cut_err((
            p_user_ident, // function name
            repeat(
                // Curried parameters
                1..,
                delim::<_, Vec<_>>('(', comma_list(0.., p_user_ident), ')'),
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
    mut curried_params: Vec<Vec<Ident>>,
    return_expr: SpanExpr,
    where_scope: Vec<(Ident, SpanExpr)>,
) -> ast::Function {
    let where_scope: Vec<(Ident, Rc<SpanExpr>)> = where_scope
        .into_iter()
        .map(|(id, expr)| (id, Rc::new(expr)))
        .collect();
    let last_params = curried_params.pop().unwrap(); // repeat(1..) guarantees Some
    // Last curried function is the main function
    let return_span = return_expr.span.clone();
    let mut acc_function = ast::Function {
        params: last_params,
        where_scope,
        return_expr: Box::new(return_expr),
    };
    let acc_span = return_span;
    // Curry remaining parameters in reverse
    // reuses the inner return span (no source position of its own).
    for params in curried_params.into_iter().rev() {
        let wrapped = Spanned {
            node: ast::Expr::Function(acc_function),
            span: acc_span.clone(),
        };
        acc_function = ast::Function {
            params,
            where_scope: vec![],
            return_expr: Box::new(wrapped),
        };
    }
    acc_function
}
