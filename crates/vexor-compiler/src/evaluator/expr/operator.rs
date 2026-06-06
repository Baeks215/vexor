use crate::evaluator::expr::{Evaluable, Value, eval, ty};
use crate::evaluator::{EResult, EnvRef};
use crate::ir::ast::{SpanExpr, op};
use crate::ir::path::{concat_paths, transform_path};
use crate::ir::scene::{Graphic, GraphicType};

/// Evaluates a binary operator expression
pub fn eval_op_bin<T: Evaluable>(
    env: &EnvRef,
    operator: op::Binary,
    left: &SpanExpr,
    right: &SpanExpr,
) -> EResult<T::Output> {
    let result = match operator {
        op::Binary::Arithmetic(operator) => {
            let l = eval::<ty::Any>(env, left)?;
            let r = eval::<ty::Any>(env, right)?;
            match (l, r) {
                (Value::Graphic(l), Value::Graphic(r)) => {
                    let Graphic {
                        ty: GraphicType::Path { path: r },
                        ..
                    } = r
                    else {
                        return Err("+ expected two paths".into());
                    };
                    Value::from(transform_path(l, |p| concat_paths(p, r))?)
                }
                (Value::Number(x), Value::Number(y)) => Value::from(match operator {
                    op::Arithmetic::Add => x + y,
                    op::Arithmetic::Sub => x - y,
                    op::Arithmetic::Mul => x * y,
                    op::Arithmetic::Div => x / y,
                    op::Arithmetic::IntDiv => (x / y).trunc(),
                    op::Arithmetic::Rem => x % y,
                    op::Arithmetic::Pow => x.powf(y),
                }),
                _ => return Err("invalid operands for +".into()),
            }
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
            let mut tail = eval::<ty::List>(env, right)?;
            tail.push_front(head); // Effective O(1)
            Value::from(tail)
        }
    };
    T::expect(result)
}

/// Evaluates a unary operator expression
pub fn eval_op_un<T: Evaluable>(
    env: &EnvRef,
    operator: op::Unary,
    expr: &SpanExpr,
) -> EResult<T::Output> {
    let result = match operator {
        op::Unary::Not => {
            let value = eval::<ty::Bool>(env, expr)?;
            Value::from(!value)
        }
        op::Unary::Neg => {
            let value = eval::<ty::Number>(env, expr)?;
            Value::from(-value)
        }
    };
    T::expect(result)
}
