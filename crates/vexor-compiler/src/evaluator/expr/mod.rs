//! Evaluator for expressions

use crate::evaluator::expr::constants::get_constant;
use crate::evaluator::{EResult, EnvExt, EnvRef};
use crate::ir::ast::{self, Expr, Literal};
use crate::ir::scene;

mod constants;
mod function;
mod list;
mod matcher;
mod operator;
mod types;

pub use function::Callable;
pub use types::*;

use function::eval_call;
use matcher::eval_match;
use operator::{eval_op_bin, eval_op_un};

/// Evaluates an expression and returns the result as the expected output type.
pub fn eval<T: Evaluable>(env: &EnvRef, expr: ast::Expr) -> EResult<T::Output> {
    match expr {
        Expr::Literal(literal) => eval_literal::<T>(env, literal),
        Expr::Variable(name) => {
            let value = env.get_var(&name)?;
            T::expect(value)
        }
        Expr::Const(c) => T::expect(get_constant(c)),
        Expr::Call { function, args } => {
            let function = eval::<ty::Function>(env, *function)?;
            let args: Vec<Value> = args
                .into_iter()
                .map(|arg_expr| eval::<ty::Any>(env, arg_expr))
                .collect::<Result<Vec<_>, _>>()?;

            eval_call::<T>(env, function, args)
        }
        Expr::Function(func) => {
            T::expect(Value::from(Callable::User {
                func,
                // Capture the current environment
                closure_env: env.clone(), // Cloned reference
            }))
        }
        Expr::Std(func) => T::expect(Value::from(Callable::Std(func))),
        Expr::Binary {
            operator,
            left,
            right,
        } => eval_op_bin::<T>(env, operator, *left, *right),
        Expr::Unary { operator, operand } => eval_op_un::<T>(env, operator, *operand),
        Expr::Match { scrutinee, arms } => {
            let s = eval::<ty::Any>(env, *scrutinee)?;
            eval_match::<T>(env, arms, s)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            if eval::<ty::Bool>(env, *condition)? {
                eval::<T>(env, *then_branch)
            } else {
                eval::<T>(env, *else_branch)
            }
        }
        Expr::Field { object, field } => eval_field_access::<T>(env, object, field),
    }
}

/// Evaluates a literal expression
fn eval_literal<T: Evaluable>(env: &EnvRef, literal: Literal) -> EResult<T::Output> {
    let result = match literal {
        Literal::Number(n) => Value::Number(n),
        Literal::String(s) => Value::String(s),
        Literal::Bool(b) => Value::Bool(b),
        Literal::Color(c) => {
            let ast::Color::Rgba { r, g, b, a } = c;
            Value::Color(scene::Color::Rgba {
                r: eval::<ty::Number>(env, *r)?,
                g: eval::<ty::Number>(env, *g)?,
                b: eval::<ty::Number>(env, *b)?,
                a: eval::<ty::Number>(env, *a)?,
            })
        }
        Literal::List(l) => Value::List(list::eval_literal(env, l)?),
    };
    T::expect(result)
}

/// Evaluates a field access expression.
fn eval_field_access<T: Evaluable>(
    env: &EnvRef,
    object: String,
    field: String,
) -> EResult<T::Output> {
    let object_value = env.get_var(&object)?;
    let result = match object_value {
        Value::Graphic(g) => match g.ty {
            scene::GraphicType::Circle { radius } => match field.as_str() {
                // TEMP: 0 for x and y
                "x" => Value::from(0.0),
                "y" => Value::from(0.0),
                "radius" => Value::from(radius),
                _ => return Err("unknown field".to_string()),
            },
            scene::GraphicType::Rect { width, height } => match field.as_str() {
                "x" => Value::from(0.0),
                "y" => Value::from(0.0),
                "width" => Value::from(width),
                "height" => Value::from(height),
                _ => return Err("unknown field".to_string()),
            },
            scene::GraphicType::Text { content } => match field.as_str() {
                "x" => Value::from(0.0),
                "y" => Value::from(0.0),
                "content" => Value::from(content),
                _ => return Err("unknown field".to_string()),
            },
            scene::GraphicType::Group { .. } => match field.as_str() {
                "x" => Value::from(0.0),
                "y" => Value::from(0.0),
                _ => return Err("unknown field".to_string()),
            },
        },
        _ => return Err("can not access field of this value".to_string()),
    };
    T::expect(result)
}
