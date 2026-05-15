use crate::evaluator::expr::{Evaluable, Value, eval, list, ty};
use crate::evaluator::{EResult, EnvRef};
use crate::ir::ast::{Expr, op};

/// Evaluates a binary operator expression
pub fn eval_op_bin<T: Evaluable>(
    env: &EnvRef,
    operator: op::Binary,
    left: Expr,
    right: Expr,
) -> EResult<T::Output> {
    let result = match operator {
        op::Binary::Arithmetic(operator) => {
            // Force evaluate as expected types
            let left = eval::<ty::Number>(env, left)?;
            let right = eval::<ty::Number>(env, right)?;
            Value::from(match operator {
                op::Arithmetic::Add => left + right,
                op::Arithmetic::Sub => left - right,
                op::Arithmetic::Mul => left * right,
                op::Arithmetic::Div => left / right,
            })
        }
        op::Binary::Logic(operator) => {
            let l = eval::<ty::Bool>(env, left)?;
            let r = eval::<ty::Bool>(env, right)?;
            Value::from(match operator {
                op::Logic::And => l && r,
                op::Logic::Or => l || r,
            })
        }
        op::Binary::Compare(operator) => {
            let l = eval::<ty::Number>(env, left)?;
            let r = eval::<ty::Number>(env, right)?;
            Value::from(match operator {
                op::Compare::Gt => l > r,
                op::Compare::Gte => l >= r,
                op::Compare::Lt => l < r,
                op::Compare::Lte => l <= r,
                op::Compare::Eq => l == r,
                op::Compare::Neq => l != r,
            })
        }
        op::Binary::Cons => {
            let head = eval::<ty::Any>(env, left)?;
            let tail = eval::<ty::List>(env, right)?;
            Value::from(Box::new(list::ListNode::Cons(head, tail)))
        }
    };
    T::expect(result)
}

/// Evaluates a unary operator expression
pub fn eval_op_un<T: Evaluable>(
    env: &EnvRef,
    operator: op::Unary,
    expr: Expr,
) -> EResult<T::Output> {
    let result = match operator {
        op::Unary::Not => {
            let value = eval::<ty::Bool>(env, expr)?;
            Value::from(!value)
        }
    };
    T::expect(result)
}
