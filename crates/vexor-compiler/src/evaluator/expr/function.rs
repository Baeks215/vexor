use itertools::Itertools;
use kurbo::Affine;

use crate::evaluator::expr::list::List;
use crate::evaluator::expr::{Evaluable, Value, eval, ty};
use crate::evaluator::{EResult, EnvExt, EnvRef};
use crate::ir::ast::{self, Function, Std};
use crate::ir::{Number, scene};
use crate::{Graphic, GraphicType};

#[derive(Debug, Clone)]
pub enum Callable {
    Std(ast::Std),
    StdLambda(StdLambda),
    User { func: Function, closure_env: EnvRef },
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
impl From<StdLambda> for Value {
    fn from(value: StdLambda) -> Self {
        Value::from(Callable::StdLambda(value))
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
        Callable::User { func, closure_env } => eval_user_call::<T>(&closure_env, func, args),
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
            let x = ty::Number::expect(x)?;
            Value::from(x.to_radians())
        }
        Std::Sin => {
            let x = unpack_1!(args)?;
            let x = ty::Number::expect(x)?;
            Value::from(x.sin())
        }
        Std::Cos => {
            let x = unpack_1!(args)?;
            let x = ty::Number::expect(x)?;
            Value::from(x.cos())
        }
        Std::Tan => {
            let x = unpack_1!(args)?;
            let x = ty::Number::expect(x)?;
            Value::from(x.tan())
        }
        // List
        Std::Map => {
            let func = unpack_1!(args)?;
            let func = Box::new(ty::Function::expect(func)?);
            // Lambda that takes in a list and applies map
            Value::from(StdLambda::Map { func })
        }
        // Graphic constructors
        Std::Circle => {
            let radius = unpack_1!(args)?;
            Value::from(Graphic::new(GraphicType::Circle {
                radius: ty::Number::expect(radius)?,
            }))
        }
        Std::Rect => {
            let (width, height) = unpack_2!(args)?;
            Value::from(Graphic::new(GraphicType::Rect {
                width: ty::Number::expect(width)?,
                height: ty::Number::expect(height)?,
            }))
        }
        Std::Text => {
            let content = unpack_1!(args)?;
            Value::from(Graphic::new(GraphicType::Text {
                content: ty::String::expect(content)?,
            }))
        }
        Std::Group => {
            let children = unpack_1!(args)?;
            let child_list = ty::List::expect(children)?;
            let children = child_list
                .into_iter()
                .map(|child| ty::Graphic::expect(child))
                .collect::<Result<Vec<_>, _>>()?;
            Value::from(Graphic::new(GraphicType::Group { children }))
        }
        // Graphic functions, as lambdas
        Std::Move => {
            let (x, y) = unpack_2!(args)?;
            let x = ty::Number::expect(x)?;
            let y = ty::Number::expect(y)?;
            // Lambda takes in a graphic and returns transformed
            Value::from(StdLambda::Move { x, y })
        }
        Std::Scale => {
            let scale = unpack_1!(args)?;
            let scale = ty::Number::expect(scale)?;
            Value::from(StdLambda::Scale { scale })
        }
        Std::Rotate => {
            let angle = unpack_1!(args)?;
            let angle = ty::Number::expect(angle)?;
            Value::from(StdLambda::Rotate { angle })
        }
        Std::Fill => {
            let color = unpack_1!(args)?;
            let color = ty::Color::expect(color)?;
            Value::from(StdLambda::Fill { color })
        }
        Std::Stroke => {
            let (width, color) = unpack_2!(args)?;
            let width = ty::Number::expect(width)?;
            let color = ty::Color::expect(color)?;
            Value::from(StdLambda::Stroke { width, color })
        }
    };
    T::expect(result)
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
            let list = ty::List::expect(list)?;

            // Evaluate and map each item
            let values = list
                .into_iter()
                .map(|item| eval_call::<ty::Any>(env, func.clone(), vec![item]))
                .collect::<Result<List, _>>()?;

            Value::List(values)
        }
        // Graphic transforms
        StdLambda::Move { x, y } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::expect(graphic)?;
            let result = graphic.transform(Affine::translate((x, y)));
            Value::from(result)
        }
        StdLambda::Scale { scale } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::expect(graphic)?;
            Value::from(graphic.transform(Affine::scale(scale)))
        }
        StdLambda::Rotate { angle } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::expect(graphic)?;
            Value::from(graphic.transform(Affine::rotate(angle)))
        }
        StdLambda::Fill { color } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::expect(graphic)?;
            Value::from(graphic.transform_style(|s| s.with_fill(color)))
        }
        StdLambda::Stroke { width, color } => {
            let graphic = unpack_1!(args)?;
            let graphic = ty::Graphic::expect(graphic)?;
            Value::from(graphic.transform_style(|s| s.with_stroke(scene::Stroke { width, color })))
        }
    };
    T::expect(result)
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
