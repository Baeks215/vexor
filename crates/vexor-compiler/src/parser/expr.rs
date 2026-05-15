//! Parser for expressions

use winnow::ascii::{dec_int, float};
use winnow::combinator::{
    Infix, Postfix, Prefix, alt, cut_err, delimited, dispatch, expression, fail, opt, peek,
    preceded, terminated,
};
use winnow::token::take_while;
use winnow::{ModalResult, Parser};

use crate::ir::Number;
use crate::ir::ast;
use crate::ir::ast::op;
use crate::parser::function::{p_lambda, p_std};
use crate::parser::{
    Input, ParserExt, comma_list, delim, delim_cut, exp_string, keyword as k, p_identifier,
};

// --- Primitives ---

/// Parses a number literal.
pub fn p_number<'a>(input: &mut Input<'a>) -> ModalResult<Number> {
    alt((
        // Integer if followed by range syntax
        //   otherwise float will take '.' off range operator
        terminated(dec_int, peek("..")).map(|n: i64| n as f64),
        float,
    ))
    .label("number")
    .ws()
    .parse_next(input)
}

/// Parses a string literal.
pub fn p_string<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    delimited(
        '"',
        take_while(0.., |c: char| c != '"'),
        cut_err('"'.expected("closing `\"`")),
    )
    .label("string")
    .ws()
    .parse_next(input)
}

/// Parses a bool literal.
pub fn p_bool<'a>(input: &mut Input<'a>) -> ModalResult<bool> {
    alt((k::pk_true.map(|_| true), k::pk_false.map(|_| false)))
        .label("bool")
        .ws()
        .parse_next(input)
}

/// Parses a color.
pub fn p_color<'a>(input: &mut Input<'a>) -> ModalResult<ast::Color> {
    preceded(
        k::pk_rgb,
        cut_err(delim(
            '(',
            comma_list(4, p_expr).map(|mut es: Vec<ast::Expr>| ast::Color::Rgba {
                r: Box::new(es.remove(0)),
                g: Box::new(es.remove(0)),
                b: Box::new(es.remove(0)),
                a: Box::new(es.remove(0)),
            }),
            ')',
        )),
    )
    .label("color")
    .ws()
    .parse_next(input)
}

/// Parses a list literal.
pub fn p_list<'a>(input: &mut Input<'a>) -> ModalResult<ast::ListLiteral> {
    alt((
        k::pk_nil.map(|_| ast::ListLiteral::List(vec![])),
        delim_cut(
            '[',
            alt((
                (
                    p_expr.mws(),
                    opt(preceded(','.mws(), cut_err(p_expr).mws())),
                    preceded("..".mws(), cut_err(p_expr)),
                )
                    .map(|(start, second, end)| ast::ListLiteral::Range {
                        start: Box::new(start),
                        second: second.map(|s| Box::new(s)),
                        end: Box::new(end),
                    }),
                comma_list(0.., p_expr).map(|es| ast::ListLiteral::List(es)),
            )),
            ']',
        ),
    ))
    .label("list")
    .expected("comma-separated list")
    .expected("range `[x..y]`")
    .expected("range `[x,y..z]`")
    .ws()
    .parse_next(input)
}

/// Parses a constant.
pub fn p_constant<'a>(input: &mut Input<'a>) -> ModalResult<ast::Const> {
    k::pk_pi.ws().parse_next(input)
}

/// Parses a literal expression.
pub fn p_literal<'a>(input: &mut Input<'a>) -> ModalResult<ast::Literal> {
    alt((
        p_number.map(ast::Literal::Number),
        p_string.map(|s| ast::Literal::String(s.to_string())),
        p_bool.map(ast::Literal::Bool),
        p_color.map(ast::Literal::Color),
        p_list.map(ast::Literal::List),
    ))
    .parse_next(input)
}

/// Parses a match arm: `<pattern> [if <guard>] => <body>`.
pub fn p_match_arm<'a>(input: &mut Input<'a>) -> ModalResult<ast::MatchArm> {
    (
        p_expr,
        opt(preceded(k::pk_if.ws(), p_expr)),
        preceded(exp_string("=>").ws(), p_expr),
    )
        .map(|(pattern, guard, body)| ast::MatchArm {
            pattern,
            guard,
            body,
        })
        .label("match arm")
        .parse_next(input)
}

/// Parses a match expression: `match <expr> { <arm>, <arm>, ... }`.
pub fn p_match<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    preceded(
        k::pk_match.ws(),
        cut_err((p_expr, delim('{', comma_list(1.., p_match_arm), '}'))),
    )
    .ws()
    .map(
        |(scrutinee, arms): (ast::Expr, Vec<ast::MatchArm>)| ast::Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        },
    )
    .parse_next(input)
}

/// Parses an if expression: `if <cond> { <then> } else { <else> }`.
pub fn p_if<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    preceded(
        k::pk_if.ws(),
        cut_err((
            p_expr,
            delim('{', p_expr, '}').ws(),
            preceded(k::pk_else.ws(), delim('{', p_expr, '}')),
        )),
    )
    .ws()
    .map(
        |(condition, then_branch, else_branch): (ast::Expr, ast::Expr, ast::Expr)| ast::Expr::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        },
    )
    .parse_next(input)
}

/// Parses identifier or object field access
pub fn p_identifier_or_field<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    (
        p_identifier.map(str::to_string),
        opt(preceded('.', p_identifier.map(str::to_string))),
    )
        .ws()
        .map(|(var, field)| match field {
            Some(field) => ast::Expr::Field { object: var, field },
            None => ast::Expr::Variable(var),
        })
        .parse_next(input)
}

fn p_tuple_or_bracketed<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    delim('(', comma_list(1.., p_expr), ')')
        .map(|mut es: Vec<ast::Expr>| {
            if es.len() == 1 {
                es.pop().unwrap()
            } else {
                ast::Expr::Literal(ast::Literal::Tuple(es))
            }
        })
        .ws()
        .parse_next(input)
}

/// Parses an atom.
pub fn p_atom<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    alt((
        p_lambda.map(ast::Expr::Function),
        p_tuple_or_bracketed,
        p_constant.map(ast::Expr::Const),
        p_std.map(ast::Expr::Std),
        p_literal.map(ast::Expr::Literal),
        p_if,
        p_match,
        p_identifier_or_field,
    ))
    .parse_next(input)
}

/// Parses an expression.
pub fn p_expr<'a>(input: &mut Input<'a>) -> ModalResult<ast::Expr> {
    expression(p_atom).infix(dispatch! {alt((
        alt((">>", "&&", "||")),
        alt(("==", "!=", ">=", "<=")),
        alt(("+", "-", "*", "/", ">", "<", ":")),
    )).ws();
        ">>" => Infix::Left(0, |_, arg, func| Ok(ast::Expr::Call { function: Box::new(func), args: vec![arg] })),
        "||" => Infix::Left(1, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Logic(op::Logic::Or), left: Box::new(a), right: Box::new(b) })),
        "&&" => Infix::Left(2, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Logic(op::Logic::And), left: Box::new(a), right: Box::new(b) })),
        // Comparisons
        "==" => Infix::Left(3, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Compare(op::Compare::Eq), left: Box::new(a), right: Box::new(b) })),
        "!=" => Infix::Left(3, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Compare(op::Compare::Neq), left: Box::new(a), right: Box::new(b) })),
        ">=" => Infix::Left(3, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Compare(op::Compare::Gte), left: Box::new(a), right: Box::new(b) })),
        "<=" => Infix::Left(3, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Compare(op::Compare::Lte), left: Box::new(a), right: Box::new(b) })),
        ">" => Infix::Left(3, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Compare(op::Compare::Gt), left: Box::new(a), right: Box::new(b) })),
        "<" => Infix::Left(3, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Compare(op::Compare::Lt), left: Box::new(a), right: Box::new(b) })),
        // Cons
        ":" => Infix::Right(4, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Cons, left: Box::new(a), right: Box::new(b) })),
        // Arithmetic
        "+" => Infix::Left(5, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Arithmetic(op::Arithmetic::Add), left: Box::new(a), right: Box::new(b) })),
        "-" => Infix::Left(5, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Arithmetic(op::Arithmetic::Sub), left: Box::new(a), right: Box::new(b) })),
        "*" => Infix::Left(7, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Arithmetic(op::Arithmetic::Mul), left: Box::new(a), right: Box::new(b) })),
        "/" => Infix::Left(7, |_, a, b| Ok(ast::Expr::Binary { operator: op::Binary::Arithmetic(op::Arithmetic::Div), left: Box::new(a), right: Box::new(b) })),
        _ => fail,
    })
    .prefix(dispatch! {"!";
        "!" => Prefix(11, |_, a| Ok(ast::Expr::Unary { operator: op::Unary::Not, operand: Box::new(a) })),
        _ => fail,
    })
    .postfix(dispatch! { peek("(");
        "(" => Postfix(12, |input, e| {
            let args: Vec<_> = delim('(',comma_list(0.., p_expr),')')
                    .ws()
                    .parse_next(input)?;
            Ok(ast::Expr::Call {
                function: Box::new(e),
                args,
            })
        }),
        _ => fail,
    })
    .parse_next(input)
}
