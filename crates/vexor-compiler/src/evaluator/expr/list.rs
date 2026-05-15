use crate::evaluator::expr::{Evaluable, Value, eval, ty};
use crate::evaluator::{EResult, EnvRef, to_int};
use crate::ir::ast::ListLiteral;

/// Linked List node
#[derive(Debug, Clone)]
pub enum ListNode<T: Clone> {
    Nil,
    Cons(T, Box<ListNode<T>>),
}

impl<T: Clone> IntoIterator for ListNode<T> {
    type Item = T;
    type IntoIter = ListIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        ListIterator { current: self }
    }
}

pub struct ListIterator<T: Clone> {
    current: ListNode<T>,
}
impl<T: Clone> Iterator for ListIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match std::mem::replace(&mut self.current, ListNode::Nil) {
            ListNode::Nil => None,
            ListNode::Cons(item, next) => {
                self.current = *next;
                Some(item)
            }
        }
    }
}

pub fn eval_literal(
    env: &EnvRef,
    literal: ListLiteral,
) -> EResult<<ty::List as Evaluable>::Output> {
    match literal {
        // Build Linked List from vector literal
        ListLiteral::List(exprs) => {
            let mut acc = Box::new(ListNode::Nil);

            // Iterate in reverse to build linked list
            for e in exprs.into_iter().rev() {
                let e = eval::<ty::Any>(env, e)?;
                acc = Box::new(ListNode::Cons(e, acc));
            }
            Ok(acc)
        }
        // Build Linked List from stepped range
        ListLiteral::Range { start, second, end } => {
            // Evaluate range bounds and convert to integers
            let start = eval::<ty::Number>(env, *start).and_then(to_int)?;
            let second = second
                .map(|e| eval::<ty::Number>(env, *e).and_then(to_int))
                .transpose()?;
            let end = eval::<ty::Number>(env, *end).and_then(to_int)?;

            let mut acc = Box::new(ListNode::Nil);

            // Iterate in reverse to build linked list
            let iter_rev = build_range_rev(start, second, end)?;
            for n in iter_rev {
                // Loss of precision for large numbers
                let value = Value::from(n as f64);
                acc = Box::new(ListNode::Cons(value, acc));
            }
            Ok(acc)
        }
    }
}

// --- Helpers --- //

/// Builds a range of integers in reverse
fn build_range_rev(
    start: i64,
    second: Option<i64>,
    end: i64,
) -> EResult<impl Iterator<Item = i64>> {
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
        return Err("range step cannot be zero.".to_string());
    }
    if start != end && total_range.signum() != step.signum() {
        return Err("range step direction is inconsistent with end.".to_string());
    }

    // Normalise end to be the last element in the range
    let end = total_range / step * step + start;

    // Switch to reverse
    let (start, end, step) = (end, start, -step);

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
