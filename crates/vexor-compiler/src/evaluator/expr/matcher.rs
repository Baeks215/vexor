use crate::evaluator::expr::constants::get_constant;
use crate::evaluator::expr::list::ListNode;
use crate::evaluator::expr::{Evaluable, Value, eval, ty};
use crate::evaluator::{EResult, EnvExt, EnvRef};
use crate::ir::ast::{self, Const, Expr, ListLiteral, Literal, MatchArm, Std, op};
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
    Err("no arm matched".to_string())
}

/// Matches an evaluated value to an expression pattern.
fn match_pattern(env: &EnvRef, scrutinee: Value, pattern: Expr) -> EResult<bool> {
    match pattern {
        Expr::Variable(name) => env.set_var(name, scrutinee).map(|_| true),
        Expr::Literal(lit_pattern) => match_literal_pattern(env, scrutinee, lit_pattern),
        Expr::Binary {
            operator,
            left,
            right,
        } => match_bin(env, scrutinee, operator, *left, *right),
        Expr::Std(std) => match_std(scrutinee, std),
        Expr::Const(c) => match_const(scrutinee, c),
        _ => Err("pattern not supported".to_string()),
    }
}

/// Matches an evaluated value to a literal expression pattern.
fn match_literal_pattern(env: &EnvRef, scrutinee: Value, pattern: Literal) -> EResult<bool> {
    match (scrutinee, pattern) {
        // Unsupported patterns
        (_, Literal::List(ListLiteral::Range { .. })) => Err("pattern not supported".to_string()),
        // Matches
        (Value::Number(s), Literal::Number(p)) => Ok(s == p),
        (Value::String(s), Literal::String(p)) => Ok(s == p),
        (Value::Bool(s), Literal::Bool(p)) => Ok(s == p),
        (Value::Color(s), Literal::Color(p)) => {
            let scene::Color::Rgba { r, g, b, a } = s;
            let ast::Color::Rgba {
                r: r_expr,
                g: g_expr,
                b: b_expr,
                a: a_expr,
            } = p;
            Ok(match_pattern(env, Value::Number(r), *r_expr)?
                && match_pattern(env, Value::Number(g), *g_expr)?
                && match_pattern(env, Value::Number(b), *b_expr)?
                && match_pattern(env, Value::Number(a), *a_expr)?)
        }
        (Value::List(s), Literal::List(ListLiteral::List(ps))) => {
            let mut node = s;
            for item_pattern in ps.into_iter() {
                let ListNode::Cons(head, tail) = *node else {
                    // Scrutinee is Nil, pattern is too long
                    return Ok(false);
                };
                let matched = match_pattern(env, head, item_pattern)?;
                if !matched {
                    return Ok(false);
                }
                node = tail;
            }
            match *node {
                ListNode::Nil => Ok(true),
                // Scrutinee still has items left, pattern is too short
                ListNode::Cons(_, _) => Ok(false),
            }
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
    left: Expr,
    right: Expr,
) -> EResult<bool> {
    match operator {
        op::Binary::Cons => {
            let Value::List(s) = scrutinee else {
                // Only lists can match cons patterns
                return Ok(false);
            };
            match *s {
                ListNode::Nil => Ok(false), // Nil cannot match cons pattern
                ListNode::Cons(head, tail) => Ok(match_pattern(env, head, left)?
                    && match_pattern(env, Value::List(tail), right)?),
            }
        }
        _ => Err("pattern not supported".to_string()),
    }
}

/// Matches an evaluated value to a standard function type pattern
fn match_std(scrutinee: Value, std: Std) -> Result<bool, String> {
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
        _ => Err("pattern not supported".to_string()),
    }
}

fn match_const(scrutinee: Value, c: Const) -> Result<bool, String> {
    match (scrutinee, get_constant(c)) {
        (Value::Number(s), Value::Number(p)) => Ok(s == p),
        (Value::String(s), Value::String(p)) => Ok(s == p),
        (Value::Bool(s), Value::Bool(p)) => Ok(s == p),
        (Value::Color(s), Value::Color(p)) => Ok(s == p),
        _ => Ok(false),
    }
}
