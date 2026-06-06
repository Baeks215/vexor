use itertools::Itertools;
use kurbo::{Affine, Point};
use std::cell::RefCell;
use std::rc::Rc;

use crate::evaluator::expr::list::List;
use crate::evaluator::expr::{Evaluable, Value, eval, ty};
use crate::evaluator::{EError, EResult, EnvExt, EnvRef, to_usize};
use crate::ir::ast::{self, Function, Std};
use crate::ir::path::{Path, catmull_rom_path, close_path, transform_path};
use crate::ir::{Number, scene};
use crate::{Graphic, GraphicType};

#[derive(Debug, Clone)]
pub enum Callable {
    Std(ast::Std),
    StdLambda(StdLambda),
    User {
        /// `Rc` so each call doesn't clone whole body.
        func: Rc<Function>,
        closure_env: EnvRef,
    },
}

#[derive(Debug, Clone)]
pub enum StdLambda {
    // List
    Map {
        func: Box<Callable>,
    },
    Filter {
        func: Box<Callable>,
    },
    FlatMap {
        func: Box<Callable>,
    },
    Find {
        func: Box<Callable>,
    },
    Drop {
        n: usize,
    },
    Take {
        n: usize,
    },
    Nth {
        index: usize,
    },
    DropWhile {
        func: Box<Callable>,
    },
    TakeWhile {
        func: Box<Callable>,
    },
    SortBy {
        func: Box<Callable>,
    },
    Zip {
        xs: List,
    },
    ZipWithFn {
        func: Box<Callable>,
    },
    ZipWithFnXs {
        func: Box<Callable>,
        xs: List,
    },
    FoldlFn {
        func: Box<Callable>,
    },
    FoldlFnInit {
        func: Box<Callable>,
        init: Box<Value>,
    },
    FoldrFn {
        func: Box<Callable>,
    },
    FoldrFnInit {
        func: Box<Callable>,
        init: Box<Value>,
    },
    // Graphic transforms
    JumpTo {
        x: Number,
        y: Number,
    },
    LineTo {
        x: Number,
        y: Number,
    },
    CurveTo {
        p1: Point,
        p2: Point,
        p3: Point,
    },
    Move {
        x: Number,
        y: Number,
    },
    Scale {
        scale: Number,
    },
    Rotate {
        angle: Number,
    },
    Fill {
        color: scene::Color,
    },
    StrokeWidth {
        width: Number,
    },
    StrokeColor {
        color: scene::Color,
    },
    StrokeJoin {
        join: scene::StrokeJoin,
    },
    StrokeCap {
        cap: scene::StrokeCap,
    },
    Opacity {
        n: Number,
    },
    SetId {
        name: String,
    },
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
macro_rules! unpack_3 {
    ($iter:expr) => {
        $iter
            .into_iter()
            .collect_tuple::<(_, _, _)>()
            .ok_or("expected three arguments")
    };
}
macro_rules! unpack_4 {
    ($iter:expr) => {
        $iter
            .into_iter()
            .collect_tuple::<(_, _, _, _)>()
            .ok_or("expected four arguments")
    };
}

fn tuple_to_numbers(v: Value) -> EResult<Vec<Number>> {
    let tuple = ty::Tuple::expect(v)?;
    tuple
        .into_vec()
        .into_iter()
        .map(ty::Number::expect)
        .collect()
}

fn tuple_to_point(v: Value) -> EResult<Point> {
    let tuple = ty::Tuple::expect(v)?;
    let (x, y) = tuple
        .into_iter()
        .collect_tuple()
        .ok_or("expected a 2-tuple (x, y)")?;
    Ok(Point::new(ty::Number::expect(x)?, ty::Number::expect(y)?))
}

/// Evaluates a standard function call.
fn eval_std_call<T: Evaluable>(
    env: &EnvRef,
    function: Std,
    args: Vec<Value>,
) -> EResult<<T as Evaluable>::Output> {
    let result = match function {
        // Trig
        Std::Rad => {
            let x = unpack_1!(args)?;
            let x = ty::Number::expect(x)?;
            Value::from(x.to_radians())
        }
        Std::Deg => {
            let x = unpack_1!(args)?;
            let x = ty::Number::expect(x)?;
            Value::from(x.to_degrees())
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
        // Math
        Std::Sinh => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.sinh())
        }
        Std::Cosh => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.cosh())
        }
        Std::Tanh => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.tanh())
        }
        Std::Asinh => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.asinh())
        }
        Std::Acosh => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.acosh())
        }
        Std::Atanh => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.atanh())
        }
        Std::Asin => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.asin())
        }
        Std::Acos => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.acos())
        }
        Std::Atan => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.atan())
        }
        Std::Atan2 => {
            let (y, x) = unpack_2!(args)?;
            let y = ty::Number::expect(y)?;
            let x = ty::Number::expect(x)?;
            Value::from(y.atan2(x))
        }
        Std::Round => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.round())
        }
        Std::Floor => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.floor())
        }
        Std::Ceil => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.ceil())
        }
        Std::Abs => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.abs())
        }
        Std::Max => {
            let (a, b) = unpack_2!(args)?;
            let a = ty::Number::expect(a)?;
            let b = ty::Number::expect(b)?;
            Value::from(a.max(b))
        }
        Std::Min => {
            let (a, b) = unpack_2!(args)?;
            let a = ty::Number::expect(a)?;
            let b = ty::Number::expect(b)?;
            Value::from(a.min(b))
        }
        Std::Clamp => {
            let (x, lo, hi) = unpack_3!(args)?;
            let x = ty::Number::expect(x)?;
            let lo = ty::Number::expect(lo)?;
            let hi = ty::Number::expect(hi)?;
            if lo > hi {
                return Err("clamp requires lo <= hi".into());
            }
            Value::from(x.max(lo).min(hi))
        }
        Std::Log => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.ln())
        }
        Std::Exp => {
            let x = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(x.exp())
        }
        // Vector functions (tuples of any arity)
        Std::Magnitude => {
            let v = tuple_to_numbers(unpack_1!(args)?)?;
            Value::from(v.into_iter().map(|c| c * c).sum::<Number>().sqrt())
        }
        Std::Normalize => {
            let v = tuple_to_numbers(unpack_1!(args)?)?;
            let mag = v.iter().map(|c| c * c).sum::<Number>().sqrt();
            if mag == 0.0 {
                return Err("cannot normalize a zero vector".into());
            }
            let out: Box<[Value]> = v.into_iter().map(|c| Value::from(c / mag)).collect();
            Value::Tuple(out)
        }
        Std::Dot => {
            let (a, b) = unpack_2!(args)?;
            let a = tuple_to_numbers(a)?;
            let b = tuple_to_numbers(b)?;
            if a.len() != b.len() {
                return Err("dot requires vectors of equal length".into());
            }
            Value::from(a.into_iter().zip(b).map(|(x, y)| x * y).sum::<Number>())
        }
        // List
        Std::Map => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::Map { func })
        }
        Std::Filter => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::Filter { func })
        }
        Std::FlatMap => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::FlatMap { func })
        }
        Std::Find => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::Find { func })
        }
        Std::Drop => {
            let n = to_usize(ty::Number::expect(unpack_1!(args)?)?)?;
            Value::from(StdLambda::Drop { n })
        }
        Std::Take => {
            let n = to_usize(ty::Number::expect(unpack_1!(args)?)?)?;
            Value::from(StdLambda::Take { n })
        }
        Std::DropWhile => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::DropWhile { func })
        }
        Std::TakeWhile => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::TakeWhile { func })
        }
        Std::SortBy => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::SortBy { func })
        }
        Std::Zip => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            Value::from(StdLambda::Zip { xs })
        }
        Std::ZipWith => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::ZipWithFn { func })
        }
        Std::Foldl => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::FoldlFn { func })
        }
        Std::Foldr => {
            let func = Box::new(ty::Callable::expect(unpack_1!(args)?)?);
            Value::from(StdLambda::FoldrFn { func })
        }
        Std::Enumerate => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            Value::List(
                xs.into_iter()
                    .enumerate()
                    .map(|(i, v)| Value::Tuple(Box::new([Value::Number(i as f64), v])))
                    .collect(),
            )
        }
        Std::Len => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            Value::from(xs.len() as Number)
        }
        Std::Reverse => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            Value::List(xs.into_iter().rev().collect())
        }
        Std::Sort => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            let mut v: Vec<Number> = xs
                .into_iter()
                .map(ty::Number::expect)
                .collect::<Result<_, _>>()?;
            v.sort_by(f64::total_cmp);
            Value::List(v.into_iter().map(Value::Number).collect())
        }
        Std::Repeat => {
            let (n, value) = unpack_2!(args)?;
            let n = to_usize(ty::Number::expect(n)?)?;
            Value::List(std::iter::repeat_n(value, n).collect())
        }
        Std::Nth => {
            let index = to_usize(ty::Number::expect(unpack_1!(args)?)?)?;
            Value::from(StdLambda::Nth { index })
        }
        Std::Head => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            xs.front().cloned().ok_or("head of empty list")?
        }
        Std::Last => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            xs.back().cloned().ok_or("last of empty list")?
        }
        Std::Tail => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            if xs.is_empty() {
                return Err("tail of empty list".into());
            }
            let (_, rest) = xs.split_at(1);
            Value::List(rest)
        }
        Std::Init => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            if xs.is_empty() {
                return Err("init of empty list".into());
            }
            let last = xs.len() - 1;
            let (rest, _) = xs.split_at(last);
            Value::List(rest)
        }
        Std::IsEmpty => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            Value::from(xs.is_empty())
        }
        Std::Sum => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            let total: Number = xs
                .into_iter()
                .map(ty::Number::expect)
                .sum::<EResult<Number>>()?;
            Value::from(total)
        }
        Std::Product => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            let total: Number = xs
                .into_iter()
                .map(ty::Number::expect)
                .product::<EResult<Number>>()?;
            Value::from(total)
        }
        Std::Concat => {
            let (xs, ys) = unpack_2!(args)?;
            let mut xs = ty::List::expect(xs)?;
            let ys = ty::List::expect(ys)?;
            xs.append(ys);
            Value::List(xs)
        }
        // Tuple
        Std::Fst => {
            let t = ty::Tuple::expect(unpack_1!(args)?)?;
            let (a, _) = t
                .into_vec()
                .into_iter()
                .collect_tuple()
                .ok_or("fst expected a 2-tuple")?;
            a
        }
        Std::Snd => {
            let t = ty::Tuple::expect(unpack_1!(args)?)?;
            let (_, b) = t
                .into_vec()
                .into_iter()
                .collect_tuple()
                .ok_or("snd expected a 2-tuple")?;
            b
        }
        // Color constructors
        Std::Rgb => {
            let (r, g, b) = unpack_3!(args)?;
            Value::from(scene::Color::Rgba {
                r: ty::Number::expect(r)?,
                g: ty::Number::expect(g)?,
                b: ty::Number::expect(b)?,
                a: 1.0,
            })
        }
        Std::Rgba => {
            let (r, g, b, a) = unpack_4!(args)?;
            Value::from(scene::Color::Rgba {
                r: ty::Number::expect(r)?,
                g: ty::Number::expect(g)?,
                b: ty::Number::expect(b)?,
                a: ty::Number::expect(a)?,
            })
        }
        Std::Hsl => {
            let (h, s, l) = unpack_3!(args)?;
            Value::from(scene::Color::Hsla {
                h: ty::Number::expect(h)?,
                s: ty::Number::expect(s)?,
                l: ty::Number::expect(l)?,
                a: 1.0,
            })
        }
        Std::Hsla => {
            let (h, s, l, a) = unpack_4!(args)?;
            Value::from(scene::Color::Hsla {
                h: ty::Number::expect(h)?,
                s: ty::Number::expect(s)?,
                l: ty::Number::expect(l)?,
                a: ty::Number::expect(a)?,
            })
        }
        // Graphic constructors
        Std::Circle => {
            let radius = unpack_1!(args)?;
            Value::from(Graphic::new(GraphicType::Circle {
                radius: ty::Number::expect(radius)?,
            }))
        }
        Std::Ellipse => {
            let (rx, ry) = unpack_2!(args)?;
            Value::from(Graphic::new(GraphicType::Ellipse {
                rx: ty::Number::expect(rx)?,
                ry: ty::Number::expect(ry)?,
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
            Value::from(Graphic::new(GraphicType::Group {
                children: children.into(),
            }))
        }
        Std::Line => {
            let (target_x, target_y) = unpack_2!(args)?;
            let target_x = ty::Number::expect(target_x)?;
            let target_y = ty::Number::expect(target_y)?;

            let mut path = Path::new();
            path.line_to(Point::new(target_x, target_y));
            Value::from(Graphic::new(GraphicType::Path { path }))
        }
        Std::Curve => {
            let (a, b, c) = unpack_3!(args)?;
            let p1 = tuple_to_point(a)?;
            let p2 = tuple_to_point(b)?;
            let p3 = tuple_to_point(c)?;
            let mut path = Path::new();
            path.curve_to(p1, p2, p3);
            Value::from(Graphic::new(GraphicType::Path { path }))
        }
        Std::Path => {
            let list = ty::List::expect(unpack_1!(args)?)?;
            let mut g = Graphic::new(GraphicType::Path { path: Path::new() });
            for item in list {
                let callable = ty::Callable::expect(item)?;
                g = eval_call::<ty::Graphic>(env, callable, vec![Value::from(g)])?;
            }
            Value::from(g)
        }
        Std::Sample => {
            let (times, f) = unpack_2!(args)?;
            let times = ty::List::expect(times)?;
            let f = ty::Callable::expect(f)?;
            let pts: Vec<Point> = times
                .into_iter()
                .map(|t| tuple_to_point(eval_call::<ty::Any>(env, f.clone(), vec![t])?))
                .collect::<Result<_, _>>()?;
            if pts.len() < 2 {
                return Err("sample requires at least 2 points".into());
            }
            let path = catmull_rom_path(&pts);
            Value::from(Graphic::new(GraphicType::Path { path }))
        }
        Std::MirrorX => {
            let g = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(g.transform_local(Affine::scale_non_uniform(1.0, -1.0)))
        }
        Std::MirrorY => {
            let g = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(g.transform_local(Affine::scale_non_uniform(-1.0, 1.0)))
        }
        // Graphic functions
        Std::Close => {
            let g = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(transform_path(g, close_path)?)
        }
        Std::JumpTo => {
            let (x, y) = unpack_2!(args)?;
            let x = ty::Number::expect(x)?;
            let y = ty::Number::expect(y)?;
            Value::from(StdLambda::JumpTo { x, y })
        }
        Std::LineTo => {
            let (x, y) = unpack_2!(args)?;
            let x = ty::Number::expect(x)?;
            let y = ty::Number::expect(y)?;
            Value::from(StdLambda::LineTo { x, y })
        }
        Std::CurveTo => {
            let (a, b, c) = unpack_3!(args)?;
            let p1 = tuple_to_point(a)?;
            let p2 = tuple_to_point(b)?;
            let p3 = tuple_to_point(c)?;
            Value::from(StdLambda::CurveTo { p1, p2, p3 })
        }
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
        Std::StrokeWidth => {
            let width = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(StdLambda::StrokeWidth { width })
        }
        Std::StrokeColor => {
            let color = ty::Color::expect(unpack_1!(args)?)?;
            Value::from(StdLambda::StrokeColor { color })
        }
        Std::StrokeJoin => {
            let kind = ty::String::expect(unpack_1!(args)?)?;
            let join = match kind.as_str() {
                "miter" => scene::StrokeJoin::Miter,
                "round" => scene::StrokeJoin::Round,
                "bevel" => scene::StrokeJoin::Bevel,
                _ => return Err("invalid stroke join, expected miter|round|bevel".into()),
            };
            Value::from(StdLambda::StrokeJoin { join })
        }
        Std::StrokeCap => {
            let kind = ty::String::expect(unpack_1!(args)?)?;
            let cap = match kind.as_str() {
                "butt" => scene::StrokeCap::Butt,
                "round" => scene::StrokeCap::Round,
                "square" => scene::StrokeCap::Square,
                _ => return Err("invalid stroke cap, expected butt|round|square".into()),
            };
            Value::from(StdLambda::StrokeCap { cap })
        }
        Std::Opacity => {
            let n = ty::Number::expect(unpack_1!(args)?)?;
            Value::from(StdLambda::Opacity { n })
        }
        Std::SetId => {
            let name = ty::String::expect(unpack_1!(args)?)?;
            Value::from(StdLambda::SetId { name })
        }
    };
    T::expect(result)
}

/// Evaluates a standard lambda call.
fn eval_std_lambda<T: Evaluable>(
    env: &EnvRef,
    function: StdLambda,
    args: Vec<Value>,
) -> EResult<<T as Evaluable>::Output> {
    let result = match function {
        StdLambda::JumpTo { x, y } => {
            let g = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(transform_path(g, |p| {
                p.move_to(Point::new(x, y));
                Ok(())
            })?)
        }
        StdLambda::LineTo { x, y } => {
            let g = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(transform_path(g, |p| {
                p.line_to(Point::new(x, y));
                Ok(())
            })?)
        }
        StdLambda::CurveTo { p1, p2, p3 } => {
            let g = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(transform_path(g, |p| {
                p.curve_to(p1, p2, p3);
                Ok(())
            })?)
        }
        StdLambda::Map { func } => {
            let func = *func;
            let list = ty::List::expect(unpack_1!(args)?)?;
            let values = list
                .into_iter()
                .map(|item| eval_call::<ty::Any>(env, func.clone(), vec![item]))
                .collect::<Result<List, _>>()?;
            Value::List(values)
        }
        StdLambda::Filter { func } => {
            let func = *func;
            let list = ty::List::expect(unpack_1!(args)?)?;
            let mut out = List::new();
            for item in list {
                if eval_call::<ty::Bool>(env, func.clone(), vec![item.clone()])? {
                    out.push_back(item);
                }
            }
            Value::List(out)
        }
        StdLambda::FlatMap { func } => {
            let func = *func;
            let list = ty::List::expect(unpack_1!(args)?)?;
            let mut out = List::new();
            for item in list {
                let sub = eval_call::<ty::List>(env, func.clone(), vec![item])?;
                out.append(sub);
            }
            Value::List(out)
        }
        StdLambda::Find { func } => {
            // Empty list if not found, otherwise singleton list with found item
            let func = *func;
            let list = ty::List::expect(unpack_1!(args)?)?;
            let mut out = List::new();
            for item in list {
                if eval_call::<ty::Bool>(env, func.clone(), vec![item.clone()])? {
                    out.push_back(item);
                    break;
                }
            }
            Value::List(out)
        }
        StdLambda::Drop { n } => {
            let list = ty::List::expect(unpack_1!(args)?)?;
            if n >= list.len() {
                Value::List(List::new())
            } else {
                let (_, right) = list.split_at(n);
                Value::List(right)
            }
        }
        StdLambda::Take { n } => {
            let list = ty::List::expect(unpack_1!(args)?)?;
            if n >= list.len() {
                Value::List(list)
            } else {
                let (left, _) = list.split_at(n);
                Value::List(left)
            }
        }
        StdLambda::Nth { index } => {
            let list = ty::List::expect(unpack_1!(args)?)?;
            list.get(index).cloned().ok_or("index out of bounds")?
        }
        StdLambda::DropWhile { func } => {
            let func = *func;
            let list = ty::List::expect(unpack_1!(args)?)?;
            let mut i = 0;
            for item in list.iter() {
                if eval_call::<ty::Bool>(env, func.clone(), vec![item.clone()])? {
                    i += 1;
                } else {
                    break;
                }
            }
            let (_, right) = list.split_at(i);
            Value::List(right)
        }
        StdLambda::TakeWhile { func } => {
            let func = *func;
            let list = ty::List::expect(unpack_1!(args)?)?;
            let mut i = 0;
            for item in list.iter() {
                if eval_call::<ty::Bool>(env, func.clone(), vec![item.clone()])? {
                    i += 1;
                } else {
                    break;
                }
            }
            let (left, _) = list.split_at(i);
            Value::List(left)
        }
        StdLambda::Zip { xs } => {
            let ys = ty::List::expect(unpack_1!(args)?)?;
            Value::List(
                xs.into_iter()
                    .zip(ys)
                    .map(|(a, b)| Value::Tuple(Box::new([a, b])))
                    .collect(),
            )
        }
        StdLambda::ZipWithFn { func } => {
            let xs = ty::List::expect(unpack_1!(args)?)?;
            Value::from(StdLambda::ZipWithFnXs { func, xs })
        }
        StdLambda::ZipWithFnXs { func, xs } => {
            let func = *func;
            let ys = ty::List::expect(unpack_1!(args)?)?;
            let result = xs
                .into_iter()
                .zip(ys)
                .map(|(a, b)| eval_call::<ty::Any>(env, func.clone(), vec![a, b]))
                .collect::<Result<List, _>>()?;
            Value::List(result)
        }
        StdLambda::FoldlFn { func } => {
            let init = Box::new(unpack_1!(args)?);
            Value::from(StdLambda::FoldlFnInit { func, init })
        }
        StdLambda::FoldlFnInit { func, init } => {
            let func = *func;
            let list = ty::List::expect(unpack_1!(args)?)?;
            let mut acc = *init;
            for item in list {
                acc = eval_call::<ty::Any>(env, func.clone(), vec![acc, item])?;
            }
            acc
        }
        StdLambda::FoldrFn { func } => {
            let init = Box::new(unpack_1!(args)?);
            Value::from(StdLambda::FoldrFnInit { func, init })
        }
        StdLambda::FoldrFnInit { func, init } => {
            let func = *func;
            let list = ty::List::expect(unpack_1!(args)?)?;
            let mut acc = *init;
            for item in list.into_iter().rev() {
                acc = eval_call::<ty::Any>(env, func.clone(), vec![item, acc])?;
            }
            acc
        }
        StdLambda::SortBy { func } => {
            let func = *func;
            let list = ty::List::expect(unpack_1!(args)?)?;
            let mut v: Vec<Value> = list.into_iter().collect();
            // Store error outside of closure
            let err: RefCell<Option<EError>> = RefCell::new(None);
            v.sort_by(|a, b| {
                if err.borrow().is_some() {
                    // Stop sorting
                    return std::cmp::Ordering::Equal;
                }
                match eval_call::<ty::Bool>(env, func.clone(), vec![a.clone(), b.clone()]) {
                    Ok(true) => std::cmp::Ordering::Less,
                    Ok(false) => {
                        match eval_call::<ty::Bool>(env, func.clone(), vec![b.clone(), a.clone()]) {
                            Ok(true) => std::cmp::Ordering::Greater,
                            Ok(false) => std::cmp::Ordering::Equal,
                            Err(e) => {
                                *err.borrow_mut() = Some(e);
                                std::cmp::Ordering::Equal
                            }
                        }
                    }
                    Err(e) => {
                        *err.borrow_mut() = Some(e);
                        std::cmp::Ordering::Equal
                    }
                }
            });
            if let Some(e) = err.into_inner() {
                return Err(e);
            }
            Value::List(v.into_iter().collect())
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
            Value::from(graphic.transform_attr(|s| s.with_fill(color)))
        }
        StdLambda::StrokeWidth { width } => {
            let graphic = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(graphic.transform_attr(|s| s.with_stroke_width(width)))
        }
        StdLambda::StrokeColor { color } => {
            let graphic = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(graphic.transform_attr(|s| s.with_stroke_color(color)))
        }
        StdLambda::StrokeJoin { join } => {
            let graphic = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(graphic.transform_attr(|s| s.with_stroke_join(join)))
        }
        StdLambda::StrokeCap { cap } => {
            let graphic = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(graphic.transform_attr(|s| s.with_stroke_cap(cap)))
        }
        StdLambda::Opacity { n } => {
            let graphic = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(graphic.transform_attr(|s| s.with_opacity(n)))
        }
        StdLambda::SetId { name } => {
            let graphic = ty::Graphic::expect(unpack_1!(args)?)?;
            Value::from(graphic.transform_attr(|a| a.with_id(name)))
        }
    };
    T::expect(result)
}

/// Evaluates a function call expression.
fn eval_user_call<T: Evaluable>(
    env: &EnvRef,
    func: Rc<Function>,
    args: Vec<Value>,
) -> EResult<T::Output> {
    // Borrow the definition out of the shared `Rc`; nothing here clones the body.
    let Function {
        params,
        where_scope,
        return_expr,
    } = &*func;
    // Ensure arguments have correct type
    if params.len() != args.len() {
        return Err(format!(
            "expected {} argument{} but got {}",
            params.len(),
            if params.len() == 1 { "" } else { "s" },
            args.len()
        )
        .into());
    }
    // Pair param name with arg values (leaf clone of the identifiers)
    let param_args: Vec<(String, Value)> = params.iter().cloned().zip(args).collect();

    // Add arguments to env as values
    let call_env = env.new_scope_function(param_args);

    // Evaluate "where" scope of values. Only non-empty for `where` functions; clones the
    // small binding expressions, not the whole body.
    for (id, value) in where_scope {
        call_env.set_var_lazy(id.clone(), value.clone())?;
    }

    // Evaluate return expression as the overall expression type.
    eval::<T>(&call_env, return_expr)
}
