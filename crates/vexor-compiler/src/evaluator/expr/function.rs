use itertools::Itertools;
use kurbo::Affine;

use crate::evaluator::expr::list::ListNode;
use crate::evaluator::expr::{Evaluable, eval};
use crate::evaluator::{EResult, EnvExt, EnvRef, Value, ty};
use crate::ir::ast::{self, Expr, Function, Literal, Std, op};
use crate::ir::{Number, scene};
use crate::{Graphic, GraphicType};

#[derive(Debug, Clone)]
pub enum Callable {
    Std(ast::Std),
    StdLambda(StdLambda),
    User(Function),
}

#[derive(Debug, Clone)]
pub enum StdLambda {
    // List
    Map { func: Box<Callable> },
    // Graphic transforms
    Move { x: Number, y: Number },
    Scale { scale: Number },
    Rotate { angle: Number },
    Fill { color: scene::Color },
    Stroke { width: Number, color: scene::Color },
}
impl From<StdLambda> for Callable {
    fn from(value: StdLambda) -> Self {
        Callable::StdLambda(value)
    }
}

impl Evaluable for ty::Function {
    type Output = Callable;
    fn to_value(value: Self::Output) -> Value {
        Value::Function(value)
    }
    fn from_value(value: Value) -> EResult<Self::Output> {
        match value {
            Value::Function(f) => Ok(f),
            _ => Err("expected a function".to_string()),
        }
    }
    fn eval_literal(_: &EnvRef, _: Literal) -> EResult<Self::Output> {
        // Currently no literal functions
        Err("expected a function".to_string())
    }
    fn match_literal(_: &EnvRef, _: Self::Output, _: Literal) -> EResult<bool> {
        Err("cannot pattern match a function".to_string())
    }
    fn match_bin(_: &EnvRef, _: Self::Output, _: op::Binary, _: Expr, _: Expr) -> EResult<bool> {
        Err("cannot pattern match a function".to_string())
    }
    fn match_call(_: &EnvRef, _: Self::Output, _: Expr, _: Vec<Expr>) -> EResult<bool> {
        Err("pattern not supported".to_string())
    }
}

/// Evaluates a function call expression.
pub fn eval_call<T: Evaluable>(
    env: &EnvRef,
    func: Callable,
    args: Vec<Value>,
) -> EResult<T::Output> {
    match func {
        Callable::Std(func) => eval_std_call::<T>(env, func, args),
        Callable::StdLambda(func) => eval_std_lambda::<T>(env, func, args),
        Callable::User(func) => eval_user_call::<T>(env, func, args),
    }
}

macro_rules! unpack_1 {
    ($iter:expr) => {
        $iter
            .into_iter()
            .exactly_one()
            .map_err(|_| "expected one argument")
    };
}
macro_rules! unpack_2 {
    ($iter:expr) => {
        $iter
            .into_iter()
            .collect_tuple::<(_, _)>()
            .ok_or("expected two arguments")
    };
}

/// Evaluates a standard function call.
fn eval_std_call<T: Evaluable>(
    _: &EnvRef,
    function: Std,
    args: Vec<Value>,
) -> Result<<T as Evaluable>::Output, String> {
    let result = match function {
        // Trig
        Std::Rad => {
            let x = unpack_1!(args)?;
            let x = ty::Number::from_value(x)?;
            Value::Number(x.to_radians())
        }
        Std::Sin => {
            let x = unpack_1!(args)?;
            let x = ty::Number::from_value(x)?;
            Value::Number(x.sin())
        }
        Std::Cos => {
            let x = unpack_1!(args)?;
            let x = ty::Number::from_value(x)?;
            Value::Number(x.cos())
        }
        Std::Tan => {
            let x = unpack_1!(args)?;
            let x = ty::Number::from_value(x)?;
            Value::Number(x.tan())
        }
        // List
        Std::Map => {
            let func = unpack_1!(args)?;
            let func = Box::new(ty::Function::from_value(func)?);
            // Lambda that takes in a list and applies map
            Value::Function(StdLambda::Map { func }.into())
        }
        // Graphic constructors
        Std::Circle => {
            let radius = unpack_1!(args)?;
            Value::Graphic(Graphic::new(GraphicType::Circle {
                radius: ty::Number::from_value(radius)?,
            }))
        }
        Std::Rect => {
            let (width, height) = unpack_2!(args)?;
            Value::Graphic(Graphic::new(GraphicType::Rect {
                width: ty::Number::from_value(width)?,
                height: ty::Number::from_value(height)?,
            }))
        }
        Std::Text => {
            let content = unpack_1!(args)?;
            Value::Graphic(Graphic::new(GraphicType::Text {
                content: ty::String::from_value(content)?,
            }))
        }
        Std::Group => {
            let children = unpack_1!(args)?;
            let child_list = ty::List::from_value(children)?;
            let children = child_list
                .into_iter()
                .map(|child| ty::Graphic::from_value(child))
                .collect::<Result<Vec<_>, _>>()?;
            Value::Graphic(Graphic::new(GraphicType::Group { children }))
        }
        // Graphic functions, as lambdas
        Std::Move => {
            let (x, y) = unpack_2!(args)?;
            let x = ty::Number::from_value(x)?;
            let y = ty::Number::from_value(y)?;
            // Lambda takes in a graphic and returns transformed
            Value::Function(StdLambda::Move { x, y }.into())
        }
        Std::Scale => {
            let scale = unpack_1!(args)?;
            let scale = ty::Number::from_value(scale)?;
            Value::Function(StdLambda::Scale { scale }.into())
        }
        Std::Rotate => {
            let angle = unpack_1!(args)?;
            let angle = ty::Number::from_value(angle)?;
            Value::Function(StdLambda::Rotate { angle }.into())
        }
        Std::Fill => {
            let color = unpack_1!(args)?;
            let color = ty::Color::from_value(color)?;
            Value::Function(StdLambda::Fill { color }.into())
        }
        Std::Stroke => {
            let (width, color) = unpack_2!(args)?;
            let width = ty::Number::from_value(width)?;
            let color = ty::Color::from_value(color)?;
            Value::Function(StdLambda::Stroke { width, color }.into())
        }
    };
    T::from_value(result)
}

/// Evaluates a standard lambda call.
fn eval_std_lambda<T: Evaluable>(
    env: &EnvRef,
    function: StdLambda,
    args: Vec<Value>,
) -> Result<<T as Evaluable>::Output, String> {
    let result = match function {
        StdLambda::Map { func } => {
            let func = *func;
            let list = unpack_1!(args)?;
            let list = ty::List::from_value(list)?;

            // Evaluate and map each item
            let values = list
                .into_iter()
                .map(|item| eval_call::<ty::Any>(env, func.clone(), vec![item]))
                .collect::<Result<Vec<_>, _>>()?;

            // Rebuild nodes in reverse order
            let mut acc = Box::new(ListNode::Nil);
            for item in values.into_iter().rev() {
                acc = Box::new(ListNode::Cons(item, acc));
            }

            Value::List(acc)
        }
        // Graphic transforms
        StdLambda::Move { x, y } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::from_value(graphic)?;
            let result = graphic.transform(Affine::translate((x, y)));
            Value::Graphic(result)
        }
        StdLambda::Scale { scale } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::from_value(graphic)?;
            Value::Graphic(graphic.transform(Affine::scale(scale)))
        }
        StdLambda::Rotate { angle } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::from_value(graphic)?;
            Value::Graphic(graphic.transform(Affine::rotate(angle)))
        }
        StdLambda::Fill { color } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::from_value(graphic)?;
            Value::Graphic(graphic.transform_style(|s| s.with_fill(color)))
        }
        StdLambda::Stroke { width, color } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::from_value(graphic)?;
            Value::Graphic(
                graphic.transform_style(|s| s.with_stroke(scene::Stroke { width, color })),
            )
        }
    };
    T::from_value(result)
}

/// Evaluates a function call expression.
fn eval_user_call<T: Evaluable>(
    env: &EnvRef,
    func: Function,
    args: Vec<Value>,
) -> EResult<T::Output> {
    let Function {
        params,
        scope,
        return_expr,
    } = func;
    // Ensure arguments have correct type
    if params.len() != args.len() {
        return Err(format!(
            "expected {} argument{} but got {}",
            params.len(),
            if params.len() == 1 { "" } else { "s" },
            args.len()
        ));
    }
    // Pair param name with arg values
    let param_args: Vec<(String, Value)> = params.into_iter().zip(args).collect();

    // Add arguments to env as variables
    let call_env = env.new_scope_function(param_args);

    // Evaluate "where" scope of variables
    for (id, value) in scope {
        call_env.set_var_lazy(id, value)?;
    }

    // Evaluate return expression as the overall expression type
    eval::<T>(&call_env, *return_expr.clone())
}
