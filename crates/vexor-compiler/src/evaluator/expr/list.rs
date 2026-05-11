use crate::evaluator::expr::{Evaluable, eval, match_pattern};
use crate::evaluator::{Context, EResult, Value, to_int, ty};
use crate::ir::ast::{Expr, ListLiteral, Literal, op};

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

impl Evaluable for ty::List {
    type Output = Box<ListNode<Value>>;
    fn to_value(value: Self::Output) -> Value {
        Value::List(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::List(l) => Ok(l),
            _ => Err("Expected a list".to_string()),
        }
    }
    fn eval_literal(context: &Context, literal: Literal) -> EResult<Self::Output> {
        match literal {
            Literal::List(list) => {
                match list {
                    // Build Linked List from vector literal
                    ListLiteral::List(exprs) => {
                        let mut acc = Box::new(ListNode::Nil);

                        // Iterate in reverse to build linked list
                        for e in exprs.into_iter().rev() {
                            let e = eval::<ty::Any>(context, e)?;
                            acc = Box::new(ListNode::Cons(e, acc));
                        }
                        Ok(acc)
                    }
                    // Build Linked List from stepped range
                    ListLiteral::Range { start, second, end } => {
                        // Evaluate range bounds and convert to integers
                        let start = eval::<ty::Number>(context, *start).and_then(to_int)?;
                        let second = second
                            .map(|e| eval::<ty::Number>(context, *e).and_then(to_int))
                            .transpose()?;
                        let end = eval::<ty::Number>(context, *end).and_then(to_int)?;

                        let mut acc = Box::new(ListNode::Nil);

                        // Iterate in reverse to build linked list
                        let iter_rev = build_range_rev(start, second, end)?;
                        for n in iter_rev {
                            // Loss of precision for large numbers
                            let value = Value::Number(n as f64);
                            acc = Box::new(ListNode::Cons(value, acc));
                        }
                        Ok(acc)
                    }
                }
            }
            _ => Err("Expected a list".to_string()),
        }
    }
    fn match_literal(
        context: &mut Context,
        scrutinee: Self::Output,
        literal_pattern: Literal,
    ) -> EResult<bool> {
        match literal_pattern {
            Literal::List(ListLiteral::List(ps)) => {
                let mut node = scrutinee;
                for item_pattern in ps.into_iter() {
                    let ListNode::Cons(head, tail) = *node else {
                        // Scrutinee is Nil, pattern is too long
                        return Ok(false);
                    };
                    let matched = match_pattern::<ty::Any>(context, head, item_pattern)?;
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
            _ => Err("Expected a list literal".to_string()),
        }
    }
    fn match_bin(
        context: &mut Context,
        scrutinee: Self::Output,
        operator: op::Binary,
        left: Expr,
        right: Expr,
    ) -> EResult<bool> {
        match operator {
            op::Binary::Cons => match *scrutinee {
                ListNode::Nil => Ok(false),
                ListNode::Cons(head, tail) => Ok(match_pattern::<ty::Any>(context, head, left)?
                    && match_pattern::<ty::List>(context, tail, right)?),
            },
            _ => Err("Pattern not supported".to_string()),
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
        return Err("Range step cannot be zero.".to_string());
    }
    if start != end && total_range.signum() != step.signum() {
        return Err("Range step direction is inconsistent with end.".to_string());
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
