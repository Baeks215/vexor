use im_rc::Vector;

use crate::evaluator::expr::{Evaluable, Value, eval, ty};
use crate::evaluator::{EResult, EnvRef, to_int};
use crate::ir::ast::ListLiteral;

/// Internal list representation
///   Persistent vector from im crate.
///   Efficient prepending and iteration.
pub type List = Vector<Value>;

pub fn eval_literal(
    env: &EnvRef,
    literal: &ListLiteral,
) -> EResult<<ty::List as Evaluable>::Output> {
    match literal {
        ListLiteral::List(exprs) => exprs
            .iter()
            .map(|e| eval::<ty::Any>(env, e))
            .collect::<Result<List, _>>(),
        ListLiteral::Range { start, second, end } => {
            // Evaluate range bounds and convert to integers
            let start = eval::<ty::Number>(env, start).and_then(to_int)?;
            let second = second
                .as_ref()
                .map(|e| eval::<ty::Number>(env, e).and_then(to_int))
                .transpose()?;
            let end = eval::<ty::Number>(env, end).and_then(to_int)?;

            // Iterate in reverse to build linked list
            let range = build_range(start, second, end)?;
            Ok(range
                .map(|n| {
                    // Accept loss of precision at extremes by casting
                    Value::from(n as f64)
                })
                .collect::<List>())
        }
    }
}

// --- Helpers --- //

/// Builds a range of integers
fn build_range(start: i64, second: Option<i64>, end: i64) -> EResult<impl Iterator<Item = i64>> {
    let step = match second {
        Some(s) => s - start,
        None => {
            if end >= start {
                1
            } else {
                -1
            }
        }
    };
    let total_range = end - start;

    // Check range step
    if step == 0 {
        return Err("range step cannot be zero.".into());
    }
    if start != end && total_range.signum() != step.signum() {
        return Err("range step direction is inconsistent with end.".into());
    }

    // Normalise end to be the last element in the range
    let end = total_range / step * step + start;

    Ok(std::iter::successors(Some(start), move |&prev| {
        let next = prev + step;
        // Check if next value is still within bounds
        if (step > 0 && next <= end) || (step < 0 && next >= end) {
            Some(next)
        } else {
            None
        }
    }))
}
