use itertools::Itertools;

use crate::evaluator::expr::constants::get_constant;
use crate::evaluator::expr::{Evaluable, Value, eval, ty};
use crate::evaluator::{EResult, EnvExt, EnvRef, WithSpan};
use crate::ir::ast::{Const, Expr, ListLiteral, Literal, MatchArm, SpanExpr, Spanned, Std, op};
use crate::ir::scene;

/// Generic match-arm evaluation.
pub fn eval_match<T: Evaluable>(
    env: &EnvRef,
    arms: Vec<MatchArm>,
    scrutinee: Value,
) -> EResult<T::Output> {
    for MatchArm {
        pattern,
        guard,
        body,
    } in arms
    {
        let mut arm_env = env.child_scope();
        let matched = match_pattern(&mut arm_env, scrutinee.clone(), pattern)?;
        if !matched {
            continue;
        }

        if let Some(condition) = guard {
            if !eval::<ty::Bool>(&arm_env, condition)? {
                continue;
            }
        }
        return eval::<T>(&arm_env, body);
    }
    Err("no arm matched".into())
}

/// Matches an evaluated value to an expression pattern.
fn match_pattern(env: &EnvRef, scrutinee: Value, pattern: SpanExpr) -> EResult<bool> {
    (match pattern.node {
        Expr::Variable(name) => env.set_var(name, scrutinee).map(|_| true),
        Expr::Literal(lit_pattern) => match_literal_pattern(env, scrutinee, lit_pattern),
        Expr::Binary {
            operator,
            left,
            right,
        } => match_bin(env, scrutinee, operator, *left, *right),
        Expr::Std(std) => match_std(scrutinee, std),
        Expr::Call { function, args } => match_call(env, scrutinee, *function, args),
        Expr::Const(c) => match_const(scrutinee, c),
        _ => Err("pattern not supported".into()),
    })
    .with_span_if_missing(pattern.span)
}

/// Matches an evaluated value to a literal expression pattern.
fn match_literal_pattern(env: &EnvRef, scrutinee: Value, pattern: Literal) -> EResult<bool> {
    match (scrutinee, pattern) {
        // Unsupported patterns
        (_, Literal::List(ListLiteral::Range { .. })) => Err("pattern not supported".into()),
        // Matches
        (Value::Number(s), Literal::Number(p)) => Ok(s == p),
        (Value::String(s), Literal::String(p)) => Ok(s == p),
        (Value::Bool(s), Literal::Bool(p)) => Ok(s == p),
        (Value::List(s), Literal::List(ListLiteral::List(ps))) => {
            if s.len() != ps.len() {
                // Lists of different lengths cannot match
                return Ok(false);
            }
            for (s_i, p_i) in s.into_iter().zip(ps) {
                if !match_pattern(env, s_i, p_i)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Tuple(s), Literal::Tuple(ps)) => {
            if s.len() != ps.len() {
                return Ok(false);
            }
            for (s_i, p_i) in s.into_iter().zip(ps) {
                if !match_pattern(env, s_i, p_i)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        // Non-matches
        _ => Ok(false),
    }
}

/// Matches an evaluated value to a binary operator pattern
fn match_bin(
    env: &EnvRef,
    scrutinee: Value,
    operator: op::Binary,
    left: SpanExpr,
    right: SpanExpr,
) -> EResult<bool> {
    match operator {
        op::Binary::Cons => {
            let Value::List(mut xs) = scrutinee else {
                // Only lists can match cons patterns
                return Ok(false);
            };
            let head = xs.pop_front(); // Effective O(1)
            match head {
                None => Ok(false), // Empty list cannot match cons pattern
                Some(head) => {
                    Ok(match_pattern(env, head, left)?
                        && match_pattern(env, Value::List(xs), right)?)
                }
            }
        }
        _ => Err("pattern not supported".into()),
    }
}

/// Matches an evaluated value to a standard function type pattern
fn match_std(scrutinee: Value, std: Std) -> EResult<bool> {
    match std {
        Std::Circle => Ok(matches!(
            scrutinee,
            Value::Graphic(scene::Graphic {
                ty: scene::GraphicType::Circle { .. },
                ..
            })
        )),
        Std::Rect => Ok(matches!(
            scrutinee,
            Value::Graphic(scene::Graphic {
                ty: scene::GraphicType::Rect { .. },
                ..
            })
        )),
        Std::Text => Ok(matches!(
            scrutinee,
            Value::Graphic(scene::Graphic {
                ty: scene::GraphicType::Text { .. },
                ..
            })
        )),
        Std::Group => Ok(matches!(
            scrutinee,
            Value::Graphic(scene::Graphic {
                ty: scene::GraphicType::Group { .. },
                ..
            })
        )),
        Std::Path => Ok(matches!(
            scrutinee,
            Value::Graphic(scene::Graphic {
                ty: scene::GraphicType::Path { .. },
                ..
            })
        )),
        _ => Err("pattern not supported".into()),
    }
}

fn match_const(scrutinee: Value, c: Const) -> EResult<bool> {
    match (scrutinee, get_constant(c)) {
        (Value::Number(s), Value::Number(p)) => Ok(s == p),
        (Value::String(s), Value::String(p)) => Ok(s == p),
        (Value::Bool(s), Value::Bool(p)) => Ok(s == p),
        (Value::Color(s), Value::Color(p)) => Ok(s == p),
        _ => Ok(false),
    }
}

fn match_call(
    env: &EnvRef,
    scrutinee: Value,
    function: SpanExpr,
    args: Vec<SpanExpr>,
) -> EResult<bool> {
    match (scrutinee, function) {
        (Value::Color(c), f) => {
            let Spanned { node: f, .. } = f;
            match f {
                Expr::Std(Std::Rgba) => {
                    let scene::Color::Rgba { r, g, b, a } = c else {
                        return Ok(false);
                    };
                    let Some((r_p, g_p, b_p, a_p)) = args.into_iter().collect_tuple() else {
                        return Ok(false);
                    };
                    Ok(match_pattern(env, Value::Number(r), r_p)?
                        && match_pattern(env, Value::Number(g), g_p)?
                        && match_pattern(env, Value::Number(b), b_p)?
                        && match_pattern(env, Value::Number(a), a_p)?)
                }
                Expr::Std(Std::Rgb) => {
                    let scene::Color::Rgba { r, g, b, a } = c else {
                        return Ok(false);
                    };
                    let Some((r_p, g_p, b_p)) = args.into_iter().collect_tuple() else {
                        return Ok(false);
                    };
                    Ok(match_pattern(env, Value::Number(r), r_p)?
                        && match_pattern(env, Value::Number(g), g_p)?
                        && match_pattern(env, Value::Number(b), b_p)?
                        && a == 1.0)
                }
                Expr::Std(Std::Hsla) => {
                    let scene::Color::Hsla { h, s, l, a } = c else {
                        return Ok(false);
                    };
                    let Some((h_p, s_p, l_p, a_p)) = args.into_iter().collect_tuple() else {
                        return Ok(false);
                    };
                    Ok(match_pattern(env, Value::Number(h), h_p)?
                        && match_pattern(env, Value::Number(s), s_p)?
                        && match_pattern(env, Value::Number(l), l_p)?
                        && match_pattern(env, Value::Number(a), a_p)?)
                }
                Expr::Std(Std::Hsl) => {
                    let scene::Color::Hsla { h, s, l, a } = c else {
                        return Ok(false);
                    };
                    let Some((h_p, s_p, l_p)) = args.into_iter().collect_tuple() else {
                        return Ok(false);
                    };
                    Ok(match_pattern(env, Value::Number(h), h_p)?
                        && match_pattern(env, Value::Number(s), s_p)?
                        && match_pattern(env, Value::Number(l), l_p)?
                        && a == 1.0)
                }
                _ => Ok(false),
            }
        }
        _ => Ok(false),
    }
}
