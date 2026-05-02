//! Evaluator for expressions

use crate::evaluator::program::eval_assignment;
use crate::evaluator::{Context, EResult, Function, Value};
use crate::ir::Number;
use crate::ir::scene;
use crate::ir::typed::expr::{
    ArithmeticOp, BoolOps, CompareOp, Expr, ExprGeneric, LogicOp, MatchArm, NumberOps, Pattern,
    SemanticType,
};
use crate::ir::typed::{self, BoolT, ColorT, GraphicT, NumberT, StringT};

pub fn eval_generic(context: &Context, expr: ExprGeneric) -> EResult<Value> {
    Ok(match expr {
        ExprGeneric::Number(expr) => Value::Number(eval_number(context, expr)?),
        ExprGeneric::String(expr) => Value::String(eval_string(context, expr)?),
        ExprGeneric::Bool(expr) => Value::Bool(eval_bool(context, expr)?),
        ExprGeneric::Color(expr) => Value::Color(eval_color(context, expr)?),
        ExprGeneric::Graphic(expr) => Value::Graphic(eval_graphic(context, expr)?),
    })
}

pub fn eval_number(context: &Context, expr: Expr<NumberT>) -> EResult<Number> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::Number(x) => Ok(x),
            _ => Err("Expected a number".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::Number(x) => Ok(x),
            _ => Err("Expected a number".to_string()),
        },
        Expr::Literal(x) => Ok(x),
        Expr::Operator(NumberOps::Arithmetic { op, left, right }) => match op {
            ArithmeticOp::Add => Ok(eval_number(context, *left)? + eval_number(context, *right)?),
            ArithmeticOp::Sub => Ok(eval_number(context, *left)? - eval_number(context, *right)?),
            ArithmeticOp::Mul => Ok(eval_number(context, *left)? * eval_number(context, *right)?),
            ArithmeticOp::Div => Ok(eval_number(context, *left)? / eval_number(context, *right)?),
        },
        Expr::Match { scrutinee, arms } => {
            let s = eval_number(context, *scrutinee)?;
            eval_match(
                context,
                arms,
                Value::Number(s),
                |ctx, lit| Ok(eval_number(ctx, lit)? == s),
                eval_number,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(context, *condition, *then_branch, *else_branch, eval_number),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_number)
        }
    }
}

/// Generic if-expression evaluation.
fn eval_if<T: SemanticType, U, F>(
    context: &Context,
    condition: Expr<BoolT>,
    then_branch: Expr<T>,
    else_branch: Expr<T>,
    eval_body: F,
) -> EResult<U>
where
    F: Fn(&Context, Expr<T>) -> EResult<U>,
{
    if eval_bool(context, condition)? {
        eval_body(context, then_branch)
    } else {
        eval_body(context, else_branch)
    }
}

/// Generic match-arm evaluation.
///   - `scrutinee_value` is wrapped as a `Value` for binding patterns.
///   - `eq_literal` checks whether a literal pattern matches the scrutinee.
///   - `eval_body` evaluates the arm body.
fn eval_match<E, U, Eq, F>(
    context: &Context,
    arms: Vec<MatchArm<E>>,
    scrutinee_value: Value,
    eq_literal: Eq,
    eval_body: F,
) -> EResult<U>
where
    Eq: Fn(&Context, E) -> EResult<bool>,
    F: Fn(&Context, E) -> EResult<U>,
{
    for MatchArm {
        pattern,
        guard,
        body,
    } in arms
    {
        let scope;
        let arm_ctx: &Context = match pattern {
            Pattern::Binding(name) => {
                scope = context.with_var(name, scrutinee_value.clone());
                &scope
            }
            Pattern::Literal(e) => {
                if !eq_literal(context, e)? {
                    continue;
                }
                context
            }
        };
        if let Some(g) = guard {
            if !eval_bool(arm_ctx, g)? {
                continue;
            }
        }
        return eval_body(arm_ctx, body);
    }
    Err("No match arm matched".to_string())
}

/// Evaluates a field access expression.
fn eval_field_access(context: &Context, object: String, field: String) -> EResult<Value> {
    let object_value = context.get_var(&object)?;
    let result = match object_value {
        Value::Graphic(g) => match g {
            scene::Graphic::Circle {
                x,
                y,
                radius,
                color,
            } => match field.as_str() {
                "x" => Value::Number(x),
                "y" => Value::Number(y),
                "radius" => Value::Number(radius),
                "color" => Value::Color(color),
                _ => return Err("Unknown field".to_string()),
            },
            scene::Graphic::Rect {
                x,
                y,
                width,
                height,
                color,
            } => match field.as_str() {
                "x" => Value::Number(x),
                "y" => Value::Number(y),
                "width" => Value::Number(width),
                "height" => Value::Number(height),
                "color" => Value::Color(color),
                _ => return Err("Unknown field".to_string()),
            },
            scene::Graphic::Text {
                x,
                y,
                content,
                color,
            } => match field.as_str() {
                "x" => Value::Number(x),
                "y" => Value::Number(y),
                "content" => Value::String(content),
                "color" => Value::Color(color),
                _ => return Err("Unknown field".to_string()),
            },
        },
        _ => return Err("Can not access field of this value".to_string()),
    };
    Ok(result)
}

pub fn eval_bool(context: &Context, expr: Expr<BoolT>) -> EResult<bool> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::Bool(x) => Ok(x),
            _ => Err("Expected a bool".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::Bool(x) => Ok(x),
            _ => Err("Expected a bool".to_string()),
        },
        Expr::Literal(b) => Ok(b),
        Expr::Operator(operation) => match operation {
            BoolOps::Compare { op, left, right } => {
                let l = eval_number(context, *left)?;
                let r = eval_number(context, *right)?;
                Ok(match op {
                    CompareOp::Gt => l > r,
                    CompareOp::Gte => l >= r,
                    CompareOp::Lt => l < r,
                    CompareOp::Lte => l <= r,
                    CompareOp::Eq => l == r,
                    CompareOp::Neq => l != r,
                })
            }
            BoolOps::Not(operand) => Ok(!eval_bool(context, *operand)?),
            BoolOps::Logic { op, left, right } => match op {
                LogicOp::And => {
                    // Short-circuit evaluation
                    if !eval_bool(context, *left)? {
                        Ok(false)
                    } else {
                        eval_bool(context, *right)
                    }
                }
                LogicOp::Or => {
                    // Short-circuit evaluation
                    if eval_bool(context, *left)? {
                        Ok(true)
                    } else {
                        eval_bool(context, *right)
                    }
                }
            },
        },
        Expr::Match { scrutinee, arms } => {
            let s = eval_bool(context, *scrutinee)?;
            eval_match(
                context,
                arms,
                Value::Bool(s),
                move |ctx, lit| Ok(eval_bool(ctx, lit)? == s),
                eval_bool,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(context, *condition, *then_branch, *else_branch, eval_bool),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_bool)
        }
    }
}

pub fn eval_string(context: &Context, expr: Expr<StringT>) -> EResult<String> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::String(x) => Ok(x),
            _ => Err("Expected a string".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::String(x) => Ok(x),
            _ => Err("Expected a string".to_string()),
        },
        Expr::Literal(s) => Ok(s),
        Expr::Operator(()) => Err("Operator not supported".to_string()),
        Expr::Match { scrutinee, arms } => {
            let s = eval_string(context, *scrutinee)?;
            let s_cmp = s.clone();
            eval_match(
                context,
                arms,
                Value::String(s),
                move |ctx, lit| Ok(eval_string(ctx, lit)? == s_cmp),
                eval_string,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(context, *condition, *then_branch, *else_branch, eval_string),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_string)
        }
    }
}

pub fn eval_color(context: &Context, expr: Expr<ColorT>) -> EResult<scene::Color> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::Color(x) => Ok(x),
            _ => Err("Expected a color".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::Color(x) => Ok(x),
            _ => Err("Expected a color".to_string()),
        },
        Expr::Literal(typed::Color::Rgba { r, g, b, a }) => Ok(scene::Color::Rgba {
            r: eval_number(context, *r)?,
            g: eval_number(context, *g)?,
            b: eval_number(context, *b)?,
            a: eval_number(context, *a)?,
        }),
        Expr::Operator(()) => Err("Operator not supported".to_string()),
        Expr::Match { scrutinee, arms } => {
            let s = eval_color(context, *scrutinee)?;
            eval_match(
                context,
                arms,
                Value::Color(s),
                move |ctx, lit| Ok(eval_color(ctx, lit)? == s),
                eval_color,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(context, *condition, *then_branch, *else_branch, eval_color),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_color)
        }
    }
}

pub fn eval_graphic(context: &Context, expr: Expr<GraphicT>) -> EResult<scene::Graphic> {
    match expr {
        Expr::Variable(name) => match context.get_var(&name)? {
            Value::Graphic(x) => Ok(x),
            _ => Err("Expected a graphic".to_string()),
        },
        Expr::Call {
            function,
            arguments,
        } => match eval_call(context, function, arguments)? {
            Value::Graphic(x) => Ok(x),
            _ => Err("Expected a graphic".to_string()),
        },
        Expr::Literal(l) => match l {
            typed::Graphic::Circle {
                x,
                y,
                radius,
                color,
            } => Ok(scene::Graphic::Circle {
                x: eval_number(context, *x)?,
                y: eval_number(context, *y)?,
                radius: eval_number(context, *radius)?,
                color: eval_color(context, *color)?,
            }),
            typed::Graphic::Rect {
                x,
                y,
                width,
                height,
                color,
            } => Ok(scene::Graphic::Rect {
                x: eval_number(context, *x)?,
                y: eval_number(context, *y)?,
                width: eval_number(context, *width)?,
                height: eval_number(context, *height)?,
                color: eval_color(context, *color)?,
            }),
            typed::Graphic::Text {
                x,
                y,
                content,
                color,
            } => Ok(scene::Graphic::Text {
                x: eval_number(context, *x)?,
                y: eval_number(context, *y)?,
                content: eval_string(context, *content)?,
                color: eval_color(context, *color)?,
            }),
        },
        Expr::Operator(()) => Err("Operator not supported".to_string()),
        Expr::Match { scrutinee, arms } => {
            let s = eval_graphic(context, *scrutinee)?;
            let s_cmp = s.clone();
            eval_match(
                context,
                arms,
                Value::Graphic(s),
                move |ctx, lit| Ok(eval_graphic(ctx, lit)? == s_cmp),
                eval_graphic,
            )
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => eval_if(
            context,
            *condition,
            *then_branch,
            *else_branch,
            eval_graphic,
        ),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_graphic)
        }
    }
}

fn eval_call(context: &Context, func: String, args: Vec<ExprGeneric>) -> EResult<Value> {
    let Function {
        params,
        scope,
        return_expr,
    } = context.get_function(&func)?;
    let mut context = context.new_scope_function(
        &params,
        args.into_iter()
            .map(|a| eval_generic(context, a))
            .collect::<Result<Vec<Value>, _>>()?,
    );

    for assignment in scope {
        eval_assignment(&mut context, assignment.clone())?;
    }
    eval_generic(&context, return_expr.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arith_expr(op: ArithmeticOp, left: Expr<NumberT>, right: Expr<NumberT>) -> Expr<NumberT> {
        Expr::Operator(NumberOps::Arithmetic {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }
    fn logic_expr(op: LogicOp, left: Expr<BoolT>, right: Expr<BoolT>) -> Expr<BoolT> {
        Expr::Operator(BoolOps::Logic {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }
    fn compare_expr(op: CompareOp, left: Expr<NumberT>, right: Expr<NumberT>) -> Expr<BoolT> {
        Expr::Operator(BoolOps::Compare {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    #[test]
    fn test_eval_number() {
        let mut context = Context::new();
        context.set_var("x".to_string(), Value::Number(10.0));

        // Literal
        let expr = Expr::Literal(42.0);
        assert_eq!(eval_number(&context, expr).unwrap(), 42.0);

        // Variable
        let expr = Expr::Variable("x".to_string());
        assert_eq!(eval_number(&context, expr).unwrap(), 10.0);

        // Binary: (x + 2) * 3  => (10 + 2) * 3 = 36
        let expr = arith_expr(
            ArithmeticOp::Mul,
            arith_expr(
                ArithmeticOp::Add,
                Expr::Variable("x".to_string()),
                Expr::Literal(2.0),
            ),
            Expr::Literal(3.0),
        );
        assert_eq!(eval_number(&context, expr).unwrap(), 36.0);
    }

    #[test]
    fn test_eval_string() {
        let mut context = Context::new();
        context.set_var("name".to_string(), Value::String("vexor".to_string()));

        let expr = Expr::Literal("hello".to_string());
        assert_eq!(eval_string(&context, expr).unwrap(), "hello");

        let expr = Expr::Variable("name".to_string());
        assert_eq!(eval_string(&context, expr).unwrap(), "vexor");
    }

    #[test]
    fn test_eval_color() {
        let context = Context::new();
        let expr = Expr::Literal(typed::Color::Rgba {
            r: Box::new(Expr::Literal(1.0)),
            g: Box::new(Expr::Literal(0.5)),
            b: Box::new(Expr::Literal(0.0)),
            a: Box::new(Expr::Literal(1.0)),
        });

        let res = eval_color(&context, expr).unwrap();
        assert_eq!(
            res,
            scene::Color::Rgba {
                r: 1.0,
                g: 0.5,
                b: 0.0,
                a: 1.0
            }
        );
    }

    fn circle_expr() -> Expr<GraphicT> {
        Expr::Literal(typed::Graphic::Circle {
            x: Box::new(Expr::Literal(0.0)),
            y: Box::new(Expr::Literal(0.0)),
            radius: Box::new(Expr::Literal(15.0)),
            color: Box::new(red_expr()),
        })
    }

    fn rect_expr() -> Expr<GraphicT> {
        Expr::Literal(typed::Graphic::Rect {
            x: Box::new(Expr::Literal(0.0)),
            y: Box::new(Expr::Literal(0.0)),
            width: Box::new(Expr::Literal(5.0)),
            height: Box::new(Expr::Literal(5.0)),
            color: Box::new(blue_expr()),
        })
    }

    #[test]
    fn test_eval_graphic() {
        let context = Context::new();

        let res = eval_graphic(&context, circle_expr()).unwrap();
        assert_eq!(
            res,
            scene::Graphic::Circle {
                x: 0.0,
                y: 0.0,
                radius: 15.0,
                color: red_scene(),
            }
        );

        let text_expr = Expr::Literal(typed::Graphic::Text {
            x: Box::new(Expr::Literal(10.0)),
            y: Box::new(Expr::Literal(20.0)),
            content: Box::new(Expr::Literal("hi".to_string())),
            color: Box::new(red_expr()),
        });
        let res = eval_graphic(&context, text_expr).unwrap();
        assert_eq!(
            res,
            scene::Graphic::Text {
                x: 10.0,
                y: 20.0,
                content: "hi".to_string(),
                color: red_scene(),
            }
        );
    }

    #[test]
    fn test_eval_generic() {
        let context = Context::new();
        let expr = ExprGeneric::Number(Expr::Literal(1.0));
        let res = eval_generic(&context, expr).unwrap();
        if let Value::Number(n) = res {
            assert_eq!(n, 1.0);
        } else {
            panic!("Expected Number");
        }

        let expr = ExprGeneric::Bool(Expr::Literal(true));
        let res = eval_generic(&context, expr).unwrap();
        if let Value::Bool(b) = res {
            assert_eq!(b, true);
        } else {
            panic!("Expected Bool");
        }
    }

    #[test]
    fn test_eval_bool_literal_and_var() {
        let mut context = Context::new();
        context.set_var("flag".to_string(), Value::Bool(true));

        let expr = Expr::Literal(false);
        assert_eq!(eval_bool(&context, expr).unwrap(), false);

        let expr = Expr::Variable("flag".to_string());
        assert_eq!(eval_bool(&context, expr).unwrap(), true);
    }

    #[test]
    fn test_eval_bool_compare() {
        let context = Context::new();
        let lit = |n: f64| Expr::Literal(n);

        let cases = [
            (CompareOp::Gt, 2.0, -1.0, true),
            (CompareOp::Gt, -1.0, -1.0, false),
            (CompareOp::Gte, -1.0, -1.0, true),
            (CompareOp::Gte, -2.0, -1.0, false),
            (CompareOp::Lt, -2.0, -1.0, true),
            (CompareOp::Lt, -1.0, -1.0, false),
            (CompareOp::Lte, -1.0, -1.0, true),
            (CompareOp::Lte, 2.0, -1.0, false),
            (CompareOp::Eq, -1.5, -1.5, true),
            (CompareOp::Eq, -1.0, 1.0, false),
            (CompareOp::Neq, -1.0, 1.0, true),
            (CompareOp::Neq, -1.0, -1.0, false),
        ];

        for (op, l, r, expected) in cases {
            let expr = compare_expr(op.clone(), lit(l), lit(r));
            assert_eq!(
                eval_bool(&context, expr).unwrap(),
                expected,
                "{:?} {} {}",
                op,
                l,
                r
            );
        }
    }

    #[test]
    fn test_eval_bool_logical() {
        let context = Context::new();
        let lit = |b: bool| Expr::Literal(b);

        let cases = [
            (LogicOp::And, true, true, true),
            (LogicOp::And, true, false, false),
            (LogicOp::And, false, true, false),
            (LogicOp::And, false, false, false),
            (LogicOp::Or, true, true, true),
            (LogicOp::Or, true, false, true),
            (LogicOp::Or, false, true, true),
            (LogicOp::Or, false, false, false),
        ];

        for (op, l, r, expected) in cases {
            let expr = logic_expr(op.clone(), lit(l), lit(r));
            assert_eq!(
                eval_bool(&context, expr).unwrap(),
                expected,
                "{:?} {} {}",
                op,
                l,
                r
            );
        }
    }

    #[test]
    fn test_eval_bool_not() {
        let context = Context::new();
        let expr = Expr::Operator(BoolOps::Not(Box::new(Expr::Literal(true))));
        assert_eq!(eval_bool(&context, expr).unwrap(), false);

        let expr = Expr::Operator(BoolOps::Not(Box::new(Expr::Literal(false))));
        assert_eq!(eval_bool(&context, expr).unwrap(), true);
    }

    #[test]
    fn test_eval_match() {
        // match x { x if x > 10 => 100, 2 => 99, y => y + 1 }
        let build = || Expr::Match {
            scrutinee: Box::new(Expr::Variable("x".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Binding("x".to_string()),
                    guard: Some(Expr::Operator(BoolOps::Compare {
                        op: CompareOp::Gt,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::Literal(10.0)),
                    })),
                    body: Expr::Literal(100.0),
                },
                MatchArm {
                    pattern: Pattern::Literal(Expr::Literal(2.0)),
                    guard: None,
                    body: Expr::Literal(99.0),
                },
                MatchArm {
                    pattern: Pattern::Binding("y".to_string()),
                    guard: None,
                    body: arith_expr(
                        ArithmeticOp::Add,
                        Expr::Variable("y".to_string()),
                        Expr::Literal(1.0),
                    ),
                },
            ],
        };

        // x=5 → binding arm wins (y=5, y+1=6)
        let mut context = Context::new();
        context.set_var("x".to_string(), Value::Number(5.0));
        assert_eq!(eval_number(&context, build()).unwrap(), 6.0);

        // x=2 → literal arm wins (99)
        let mut context = Context::new();
        context.set_var("x".to_string(), Value::Number(2.0));
        assert_eq!(eval_number(&context, build()).unwrap(), 99.0);

        // x=20 → guard arm wins (100)
        let mut context = Context::new();
        context.set_var("x".to_string(), Value::Number(20.0));
        assert_eq!(eval_number(&context, build()).unwrap(), 100.0);
    }

    #[test]
    fn test_eval_match_no_match() {
        // match 5 { 0 => 1 } — no arm matches.
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Literal(5.0)),
            arms: vec![MatchArm {
                pattern: Pattern::Literal(Expr::Literal(0.0)),
                guard: None,
                body: Expr::Literal(1.0),
            }],
        };
        let context = Context::new();
        assert!(eval_number(&context, expr).is_err());
    }

    #[test]
    fn test_eval_match_guard_sees_binding() {
        // match 5 { n if n == 5 => n * 2 } → 10
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Literal(5.0)),
            arms: vec![MatchArm {
                pattern: Pattern::Binding("n".to_string()),
                guard: Some(Expr::Operator(BoolOps::Compare {
                    op: CompareOp::Eq,
                    left: Box::new(Expr::Variable("n".to_string())),
                    right: Box::new(Expr::Literal(5.0)),
                })),
                body: arith_expr(
                    ArithmeticOp::Mul,
                    Expr::Variable("n".to_string()),
                    Expr::Literal(2.0),
                ),
            }],
        };
        let context = Context::new();
        assert_eq!(eval_number(&context, expr).unwrap(), 10.0);
    }

    #[test]
    fn test_eval_if_true() {
        // if true { 100 } else { 200 } → 100
        let context = Context::new();
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(true)),
            then_branch: Box::new(Expr::Literal(100.0)),
            else_branch: Box::new(Expr::Literal(200.0)),
        };
        assert_eq!(eval_number(&context, expr).unwrap(), 100.0);
    }

    #[test]
    fn test_eval_if_false() {
        // if false { 100 } else { 200 } → 200
        let context = Context::new();
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(false)),
            then_branch: Box::new(Expr::Literal(100.0)),
            else_branch: Box::new(Expr::Literal(200.0)),
        };
        assert_eq!(eval_number(&context, expr).unwrap(), 200.0);
    }

    #[test]
    fn test_eval_if_short_circuit() {
        // Unchosen branch must not evaluate.
        let context = Context::new();
        let bad = Box::new(Expr::Call {
            function: "nonexistent".to_string(),
            arguments: vec![],
        });
        // if true { 1 } else { <bad> } → 1
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(true)),
            then_branch: Box::new(Expr::Literal(1.0)),
            else_branch: bad,
        };
        assert_eq!(eval_number(&context, expr).unwrap(), 1.0);

        let bad = Box::new(Expr::Call {
            function: "nonexistent".to_string(),
            arguments: vec![],
        });
        // if false { <bad> } else { 2 } → 2
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(false)),
            then_branch: bad,
            else_branch: Box::new(Expr::Literal(2.0)),
        };
        assert_eq!(eval_number(&context, expr).unwrap(), 2.0);
    }

    #[test]
    fn test_eval_if_string_and_bool() {
        let context = Context::new();
        // string
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(true)),
            then_branch: Box::new(Expr::Literal("yes".to_string())),
            else_branch: Box::new(Expr::Literal("no".to_string())),
        };
        assert_eq!(eval_string(&context, expr).unwrap(), "yes");

        // bool
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(false)),
            then_branch: Box::new(Expr::Literal(true)),
            else_branch: Box::new(Expr::Literal(false)),
        };
        assert_eq!(eval_bool(&context, expr).unwrap(), false);
    }

    #[test]
    fn test_eval_bool_short_circuit() {
        let context = Context::new();
        // false && <bad call> — RHS must not run
        let bad = Box::new(Expr::Call {
            function: "nonexistent".to_string(),
            arguments: vec![],
        });
        let expr = logic_expr(LogicOp::And, Expr::Literal(false), *bad.clone());
        assert_eq!(eval_bool(&context, expr).unwrap(), false);

        // true || <bad call> — RHS must not run
        let bad = Box::new(Expr::Call {
            function: "nonexistent".to_string(),
            arguments: vec![],
        });
        let expr = logic_expr(LogicOp::Or, Expr::Literal(true), *bad.clone());
        assert_eq!(eval_bool(&context, expr).unwrap(), true);
    }

    // --- if/match for Color / Graphic ---

    fn red_expr() -> Expr<ColorT> {
        Expr::Literal(typed::Color::Rgba {
            r: Box::new(Expr::Literal(1.0)),
            g: Box::new(Expr::Literal(0.0)),
            b: Box::new(Expr::Literal(0.0)),
            a: Box::new(Expr::Literal(1.0)),
        })
    }

    fn blue_expr() -> Expr<ColorT> {
        Expr::Literal(typed::Color::Rgba {
            r: Box::new(Expr::Literal(0.0)),
            g: Box::new(Expr::Literal(0.0)),
            b: Box::new(Expr::Literal(1.0)),
            a: Box::new(Expr::Literal(1.0)),
        })
    }

    fn red_scene() -> scene::Color {
        scene::Color::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    fn blue_scene() -> scene::Color {
        scene::Color::Rgba {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    #[test]
    fn test_eval_if_color() {
        let context = Context::new();
        // if true { red } else { blue } → red
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(true)),
            then_branch: Box::new(red_expr()),
            else_branch: Box::new(blue_expr()),
        };
        assert_eq!(eval_color(&context, expr).unwrap(), red_scene());

        // if false { red } else { blue } → blue
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(false)),
            then_branch: Box::new(red_expr()),
            else_branch: Box::new(blue_expr()),
        };
        assert_eq!(eval_color(&context, expr).unwrap(), blue_scene());
    }

    #[test]
    fn test_eval_match_color_literal() {
        let context = Context::new();
        // match red { red => blue, x => x } → blue
        let expr = Expr::Match {
            scrutinee: Box::new(red_expr()),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(red_expr()),
                    guard: None,
                    body: blue_expr(),
                },
                MatchArm {
                    pattern: Pattern::Binding("x".to_string()),
                    guard: None,
                    body: Expr::Variable("x".to_string()),
                },
            ],
        };
        assert_eq!(eval_color(&context, expr).unwrap(), blue_scene());
    }

    #[test]
    fn test_eval_match_color_binding() {
        let context = Context::new();
        // match red { blue => red, x => x } → red (binding arm wins)
        let expr = Expr::Match {
            scrutinee: Box::new(red_expr()),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(blue_expr()),
                    guard: None,
                    body: red_expr(),
                },
                MatchArm {
                    pattern: Pattern::Binding("x".to_string()),
                    guard: None,
                    body: Expr::Variable("x".to_string()),
                },
            ],
        };
        assert_eq!(eval_color(&context, expr).unwrap(), red_scene());
    }

    #[test]
    fn test_eval_match_color_no_match() {
        let context = Context::new();
        // match red { blue => red } → err
        let expr = Expr::Match {
            scrutinee: Box::new(red_expr()),
            arms: vec![MatchArm {
                pattern: Pattern::Literal(blue_expr()),
                guard: None,
                body: red_expr(),
            }],
        };
        assert!(eval_color(&context, expr).is_err());
    }

    #[test]
    fn test_eval_if_graphic() {
        let context = Context::new();
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(false)),
            then_branch: Box::new(circle_expr()),
            else_branch: Box::new(rect_expr()),
        };
        assert_eq!(
            eval_graphic(&context, expr).unwrap(),
            scene::Graphic::Rect {
                x: 0.0,
                y: 0.0,
                width: 5.0,
                height: 5.0,
                color: blue_scene(),
            }
        );
    }

    #[test]
    fn test_eval_match_graphic() {
        let context = Context::new();
        // match circle { circle => rect } → rect
        let expr = Expr::Match {
            scrutinee: Box::new(circle_expr()),
            arms: vec![MatchArm {
                pattern: Pattern::Literal(circle_expr()),
                guard: None,
                body: rect_expr(),
            }],
        };
        assert_eq!(
            eval_graphic(&context, expr).unwrap(),
            scene::Graphic::Rect {
                x: 0.0,
                y: 0.0,
                width: 5.0,
                height: 5.0,
                color: blue_scene(),
            }
        );
    }

    #[test]
    fn test_eval_field_access() {
        let mut context = Context::new();
        context.set_var(
            "circle".to_string(),
            Value::Graphic(scene::Graphic::Circle {
                x: 2.0,
                y: 3.0,
                radius: 5.0,
                color: blue_scene(),
            }),
        );
        let expr = Expr::Field {
            object: "circle".to_string(),
            field: "x".to_string(),
        };
        assert_eq!(eval_number(&context, expr).unwrap(), 2.0);

        let expr = Expr::Field {
            object: "circle".to_string(),
            field: "color".to_string(),
        };
        assert_eq!(eval_color(&context, expr).unwrap(), blue_scene());

        let expr = Expr::Field {
            object: "circle".to_string(),
            field: "not_a_field".to_string(),
        };
        assert!(eval_number(&context, expr).is_err());
    }
}
