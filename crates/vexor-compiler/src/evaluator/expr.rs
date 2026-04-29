//! Evaluator for expressions

use crate::evaluator::program::eval_assignment;
use crate::evaluator::{Context, EResult, Function, Value};
use crate::ir::Number;
use crate::ir::scene;
use crate::ir::typed;
use crate::ir::typed::expr::{
    Expr, ExprBool, ExprColor, ExprGeneric, ExprGraphic, ExprNumber, ExprString, If, MatchArm,
    NodeBool, NodeColor, NodeGraphic, NodeNumber, NodeString, OpBinBool, OpBinNumber, OpCompare,
    OpUnBool, Pattern,
};

pub fn eval_generic(context: &Context, expr: ExprGeneric) -> EResult<Value> {
    Ok(match expr {
        ExprGeneric::Number(expr) => Value::Number(eval_number(context, expr)?),
        ExprGeneric::String(expr) => Value::String(eval_string(context, expr)?),
        ExprGeneric::Bool(expr) => Value::Bool(eval_bool(context, expr)?),
        ExprGeneric::Color(expr) => Value::Color(eval_color(context, expr)?),
        ExprGeneric::Graphic(expr) => Value::Graphic(eval_graphic(context, expr)?),
    })
}

pub fn eval_number(context: &Context, expr: ExprNumber) -> EResult<Number> {
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
        Expr::Node(NodeNumber::Literal(x)) => Ok(x),
        Expr::Node(NodeNumber::Binary {
            operator,
            left,
            right,
        }) => match operator {
            OpBinNumber::Add => Ok(eval_number(context, *left)? + eval_number(context, *right)?),
            OpBinNumber::Sub => Ok(eval_number(context, *left)? - eval_number(context, *right)?),
            OpBinNumber::Mul => Ok(eval_number(context, *left)? * eval_number(context, *right)?),
            OpBinNumber::Div => Ok(eval_number(context, *left)? / eval_number(context, *right)?),
        },
        Expr::Node(NodeNumber::Match { scrutinee, arms }) => {
            let s = eval_number(context, *scrutinee)?;
            eval_match(
                context,
                arms,
                Value::Number(s),
                |ctx, lit| Ok(eval_number(ctx, lit)? == s),
                eval_number,
            )
        }
        Expr::Node(NodeNumber::If(if_)) => eval_if(context, if_, eval_number),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_number)
        }
    }
}

/// Generic if-expression evaluation.
fn eval_if<E, U, F>(context: &Context, if_: If<E>, eval_body: F) -> EResult<U>
where
    F: Fn(&Context, E) -> EResult<U>,
{
    let If {
        condition,
        then_branch,
        else_branch,
    } = if_;
    if eval_bool(context, *condition)? {
        eval_body(context, *then_branch)
    } else {
        eval_body(context, *else_branch)
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

pub fn eval_bool(context: &Context, expr: ExprBool) -> EResult<bool> {
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
        Expr::Node(NodeBool::Literal(b)) => Ok(b),
        Expr::Node(NodeBool::Compare {
            operator,
            left,
            right,
        }) => {
            let l = eval_number(context, *left)?;
            let r = eval_number(context, *right)?;
            Ok(match operator {
                OpCompare::Gt => l > r,
                OpCompare::Gte => l >= r,
                OpCompare::Lt => l < r,
                OpCompare::Lte => l <= r,
                OpCompare::Eq => l == r,
                OpCompare::Neq => l != r,
            })
        }
        Expr::Node(NodeBool::Unary {
            operator: OpUnBool::Not,
            operand,
        }) => Ok(!eval_bool(context, *operand)?),
        Expr::Node(NodeBool::Binary {
            operator,
            left,
            right,
        }) => match operator {
            OpBinBool::And => {
                // Short-circuit evaluation
                if !eval_bool(context, *left)? {
                    Ok(false)
                } else {
                    eval_bool(context, *right)
                }
            }
            OpBinBool::Or => {
                // Short-circuit evaluation
                if eval_bool(context, *left)? {
                    Ok(true)
                } else {
                    eval_bool(context, *right)
                }
            }
            OpBinBool::Eq => Ok(eval_bool(context, *left)? == eval_bool(context, *right)?),
            OpBinBool::Neq => Ok(eval_bool(context, *left)? != eval_bool(context, *right)?),
        },
        Expr::Node(NodeBool::Match { scrutinee, arms }) => {
            let s = eval_bool(context, *scrutinee)?;
            eval_match(
                context,
                arms,
                Value::Bool(s),
                move |ctx, lit| Ok(eval_bool(ctx, lit)? == s),
                eval_bool,
            )
        }
        Expr::Node(NodeBool::If(if_)) => eval_if(context, if_, eval_bool),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_bool)
        }
    }
}

pub fn eval_string(context: &Context, expr: ExprString) -> EResult<String> {
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
        Expr::Node(NodeString::Literal(s)) => Ok(s),
        Expr::Node(NodeString::Match { scrutinee, arms }) => {
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
        Expr::Node(NodeString::If(if_)) => eval_if(context, if_, eval_string),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_string)
        }
    }
}

pub fn eval_color(context: &Context, expr: ExprColor) -> EResult<scene::Color> {
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
        Expr::Node(NodeColor::Literal(typed::Color::Rgba { r, g, b, a })) => {
            Ok(scene::Color::Rgba {
                r: eval_number(context, *r)?,
                g: eval_number(context, *g)?,
                b: eval_number(context, *b)?,
                a: eval_number(context, *a)?,
            })
        }
        Expr::Node(NodeColor::Match { scrutinee, arms }) => {
            let s = eval_color(context, *scrutinee)?;
            eval_match(
                context,
                arms,
                Value::Color(s),
                move |ctx, lit| Ok(eval_color(ctx, lit)? == s),
                eval_color,
            )
        }
        Expr::Node(NodeColor::If(if_)) => eval_if(context, if_, eval_color),
        Expr::Field { object, field } => {
            eval_field_access(context, object, field).and_then(Value::as_color)
        }
    }
}

pub fn eval_graphic(context: &Context, expr: ExprGraphic) -> EResult<scene::Graphic> {
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
        Expr::Node(NodeGraphic::Literal(typed::Graphic::Circle {
            x,
            y,
            radius,
            color,
        })) => Ok(scene::Graphic::Circle {
            x: eval_number(context, *x)?,
            y: eval_number(context, *y)?,
            radius: eval_number(context, *radius)?,
            color: eval_color(context, *color)?,
        }),
        Expr::Node(NodeGraphic::Literal(typed::Graphic::Rect {
            x,
            y,
            width,
            height,
            color,
        })) => Ok(scene::Graphic::Rect {
            x: eval_number(context, *x)?,
            y: eval_number(context, *y)?,
            width: eval_number(context, *width)?,
            height: eval_number(context, *height)?,
            color: eval_color(context, *color)?,
        }),
        Expr::Node(NodeGraphic::Literal(typed::Graphic::Text {
            x,
            y,
            content,
            color,
        })) => Ok(scene::Graphic::Text {
            x: eval_number(context, *x)?,
            y: eval_number(context, *y)?,
            content: eval_string(context, *content)?,
            color: eval_color(context, *color)?,
        }),
        Expr::Node(NodeGraphic::Match { scrutinee, arms }) => {
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
        Expr::Node(NodeGraphic::If(if_)) => eval_if(context, if_, eval_graphic),
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

    #[test]
    fn test_eval_number() {
        let mut context = Context::new();
        context.set_var("x".to_string(), Value::Number(10.0));

        // Literal
        let expr = Expr::Node(NodeNumber::Literal(42.0));
        assert_eq!(eval_number(&context, expr).unwrap(), 42.0);

        // Variable
        let expr = Expr::Variable("x".to_string());
        assert_eq!(eval_number(&context, expr).unwrap(), 10.0);

        // Binary: (x + 2) * 3  => (10 + 2) * 3 = 36
        let expr = Expr::Node(NodeNumber::Binary {
            operator: OpBinNumber::Mul,
            left: Box::new(Expr::Node(NodeNumber::Binary {
                operator: OpBinNumber::Add,
                left: Box::new(Expr::Variable("x".to_string())),
                right: Box::new(Expr::Node(NodeNumber::Literal(2.0))),
            })),
            right: Box::new(Expr::Node(NodeNumber::Literal(3.0))),
        });
        assert_eq!(eval_number(&context, expr).unwrap(), 36.0);
    }

    #[test]
    fn test_eval_string() {
        let mut context = Context::new();
        context.set_var("name".to_string(), Value::String("vexor".to_string()));

        let expr = Expr::Node(NodeString::Literal("hello".to_string()));
        assert_eq!(eval_string(&context, expr).unwrap(), "hello");

        let expr = Expr::Variable("name".to_string());
        assert_eq!(eval_string(&context, expr).unwrap(), "vexor");
    }

    #[test]
    fn test_eval_color() {
        let context = Context::new();
        let expr = Expr::Node(NodeColor::Literal(typed::Color::Rgba {
            r: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
            g: Box::new(Expr::Node(NodeNumber::Literal(0.5))),
            b: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            a: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
        }));

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

    fn circle_expr() -> ExprGraphic {
        Expr::Node(NodeGraphic::Literal(typed::Graphic::Circle {
            x: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            y: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            radius: Box::new(Expr::Node(NodeNumber::Literal(15.0))),
            color: Box::new(red_expr()),
        }))
    }

    fn rect_expr() -> ExprGraphic {
        Expr::Node(NodeGraphic::Literal(typed::Graphic::Rect {
            x: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            y: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            width: Box::new(Expr::Node(NodeNumber::Literal(5.0))),
            height: Box::new(Expr::Node(NodeNumber::Literal(5.0))),
            color: Box::new(blue_expr()),
        }))
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

        let text_expr = Expr::Node(NodeGraphic::Literal(typed::Graphic::Text {
            x: Box::new(Expr::Node(NodeNumber::Literal(10.0))),
            y: Box::new(Expr::Node(NodeNumber::Literal(20.0))),
            content: Box::new(Expr::Node(NodeString::Literal("hi".to_string()))),
            color: Box::new(red_expr()),
        }));
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
        let expr = ExprGeneric::Number(Expr::Node(NodeNumber::Literal(1.0)));
        let res = eval_generic(&context, expr).unwrap();
        if let Value::Number(n) = res {
            assert_eq!(n, 1.0);
        } else {
            panic!("Expected Number");
        }

        let expr = ExprGeneric::Bool(Expr::Node(NodeBool::Literal(true)));
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

        let expr = Expr::Node(NodeBool::Literal(false));
        assert_eq!(eval_bool(&context, expr).unwrap(), false);

        let expr = Expr::Variable("flag".to_string());
        assert_eq!(eval_bool(&context, expr).unwrap(), true);
    }

    #[test]
    fn test_eval_bool_compare() {
        let context = Context::new();
        let lit = |n: f64| Box::new(Expr::Node(NodeNumber::Literal(n)));

        let cases = [
            (OpCompare::Gt, 2.0, -1.0, true),
            (OpCompare::Gt, -1.0, -1.0, false),
            (OpCompare::Gte, -1.0, -1.0, true),
            (OpCompare::Gte, -2.0, -1.0, false),
            (OpCompare::Lt, -2.0, -1.0, true),
            (OpCompare::Lt, -1.0, -1.0, false),
            (OpCompare::Lte, -1.0, -1.0, true),
            (OpCompare::Lte, 2.0, -1.0, false),
            (OpCompare::Eq, -1.5, -1.5, true),
            (OpCompare::Eq, -1.0, 1.0, false),
            (OpCompare::Neq, -1.0, 1.0, true),
            (OpCompare::Neq, -1.0, -1.0, false),
        ];

        for (op, l, r, expected) in cases {
            let expr = Expr::Node(NodeBool::Compare {
                operator: op,
                left: lit(l),
                right: lit(r),
            });
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
        let blit = |b: bool| Box::new(Expr::Node(NodeBool::Literal(b)));

        let cases = [
            (OpBinBool::And, true, true, true),
            (OpBinBool::And, true, false, false),
            (OpBinBool::And, false, true, false),
            (OpBinBool::And, false, false, false),
            (OpBinBool::Or, true, true, true),
            (OpBinBool::Or, true, false, true),
            (OpBinBool::Or, false, true, true),
            (OpBinBool::Or, false, false, false),
            (OpBinBool::Eq, true, true, true),
            (OpBinBool::Eq, true, false, false),
            (OpBinBool::Neq, true, false, true),
            (OpBinBool::Neq, true, true, false),
        ];

        for (op, l, r, expected) in cases {
            let expr = Expr::Node(NodeBool::Binary {
                operator: op,
                left: blit(l),
                right: blit(r),
            });
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
        let expr = Expr::Node(NodeBool::Unary {
            operator: OpUnBool::Not,
            operand: Box::new(Expr::Node(NodeBool::Literal(true))),
        });
        assert_eq!(eval_bool(&context, expr).unwrap(), false);

        let expr = Expr::Node(NodeBool::Unary {
            operator: OpUnBool::Not,
            operand: Box::new(Expr::Node(NodeBool::Literal(false))),
        });
        assert_eq!(eval_bool(&context, expr).unwrap(), true);
    }

    #[test]
    fn test_eval_match() {
        // match x { x if x > 10 => 100, 2 => 99, y => y + 1 }
        let build = || {
            Expr::Node(NodeNumber::Match {
                scrutinee: Box::new(Expr::Variable("x".to_string())),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Binding("x".to_string()),
                        guard: Some(Expr::Node(NodeBool::Compare {
                            operator: OpCompare::Gt,
                            left: Box::new(Expr::Variable("x".to_string())),
                            right: Box::new(Expr::Node(NodeNumber::Literal(10.0))),
                        })),
                        body: Expr::Node(NodeNumber::Literal(100.0)),
                    },
                    MatchArm {
                        pattern: Pattern::Literal(Expr::Node(NodeNumber::Literal(2.0))),
                        guard: None,
                        body: Expr::Node(NodeNumber::Literal(99.0)),
                    },
                    MatchArm {
                        pattern: Pattern::Binding("y".to_string()),
                        guard: None,
                        body: Expr::Node(NodeNumber::Binary {
                            operator: OpBinNumber::Add,
                            left: Box::new(Expr::Variable("y".to_string())),
                            right: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
                        }),
                    },
                ],
            })
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
        let expr = Expr::Node(NodeNumber::Match {
            scrutinee: Box::new(Expr::Node(NodeNumber::Literal(5.0))),
            arms: vec![MatchArm {
                pattern: Pattern::Literal(Expr::Node(NodeNumber::Literal(0.0))),
                guard: None,
                body: Expr::Node(NodeNumber::Literal(1.0)),
            }],
        });
        let context = Context::new();
        assert!(eval_number(&context, expr).is_err());
    }

    #[test]
    fn test_eval_match_guard_sees_binding() {
        // match 5 { n if n == 5 => n * 2 } → 10
        let expr = Expr::Node(NodeNumber::Match {
            scrutinee: Box::new(Expr::Node(NodeNumber::Literal(5.0))),
            arms: vec![MatchArm {
                pattern: Pattern::Binding("n".to_string()),
                guard: Some(Expr::Node(NodeBool::Compare {
                    operator: OpCompare::Eq,
                    left: Box::new(Expr::Variable("n".to_string())),
                    right: Box::new(Expr::Node(NodeNumber::Literal(5.0))),
                })),
                body: Expr::Node(NodeNumber::Binary {
                    operator: OpBinNumber::Mul,
                    left: Box::new(Expr::Variable("n".to_string())),
                    right: Box::new(Expr::Node(NodeNumber::Literal(2.0))),
                }),
            }],
        });
        let context = Context::new();
        assert_eq!(eval_number(&context, expr).unwrap(), 10.0);
    }

    #[test]
    fn test_eval_if_true() {
        // if true { 100 } else { 200 } → 100
        let context = Context::new();
        let expr = Expr::Node(NodeNumber::If(If {
            condition: Box::new(Expr::Node(NodeBool::Literal(true))),
            then_branch: Box::new(Expr::Node(NodeNumber::Literal(100.0))),
            else_branch: Box::new(Expr::Node(NodeNumber::Literal(200.0))),
        }));
        assert_eq!(eval_number(&context, expr).unwrap(), 100.0);
    }

    #[test]
    fn test_eval_if_false() {
        // if false { 100 } else { 200 } → 200
        let context = Context::new();
        let expr = Expr::Node(NodeNumber::If(If {
            condition: Box::new(Expr::Node(NodeBool::Literal(false))),
            then_branch: Box::new(Expr::Node(NodeNumber::Literal(100.0))),
            else_branch: Box::new(Expr::Node(NodeNumber::Literal(200.0))),
        }));
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
        let expr = Expr::Node(NodeNumber::If(If {
            condition: Box::new(Expr::Node(NodeBool::Literal(true))),
            then_branch: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
            else_branch: bad,
        }));
        assert_eq!(eval_number(&context, expr).unwrap(), 1.0);

        let bad = Box::new(Expr::Call {
            function: "nonexistent".to_string(),
            arguments: vec![],
        });
        // if false { <bad> } else { 2 } → 2
        let expr = Expr::Node(NodeNumber::If(If {
            condition: Box::new(Expr::Node(NodeBool::Literal(false))),
            then_branch: bad,
            else_branch: Box::new(Expr::Node(NodeNumber::Literal(2.0))),
        }));
        assert_eq!(eval_number(&context, expr).unwrap(), 2.0);
    }

    #[test]
    fn test_eval_if_string_and_bool() {
        let context = Context::new();
        // string
        let expr = Expr::Node(NodeString::If(If {
            condition: Box::new(Expr::Node(NodeBool::Literal(true))),
            then_branch: Box::new(Expr::Node(NodeString::Literal("yes".to_string()))),
            else_branch: Box::new(Expr::Node(NodeString::Literal("no".to_string()))),
        }));
        assert_eq!(eval_string(&context, expr).unwrap(), "yes");

        // bool
        let expr = Expr::Node(NodeBool::If(If {
            condition: Box::new(Expr::Node(NodeBool::Literal(false))),
            then_branch: Box::new(Expr::Node(NodeBool::Literal(true))),
            else_branch: Box::new(Expr::Node(NodeBool::Literal(false))),
        }));
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
        let expr = Expr::Node(NodeBool::Binary {
            operator: OpBinBool::And,
            left: Box::new(Expr::Node(NodeBool::Literal(false))),
            right: bad,
        });
        assert_eq!(eval_bool(&context, expr).unwrap(), false);

        // true || <bad call> — RHS must not run
        let bad = Box::new(Expr::Call {
            function: "nonexistent".to_string(),
            arguments: vec![],
        });
        let expr = Expr::Node(NodeBool::Binary {
            operator: OpBinBool::Or,
            left: Box::new(Expr::Node(NodeBool::Literal(true))),
            right: bad,
        });
        assert_eq!(eval_bool(&context, expr).unwrap(), true);
    }

    // --- if/match for Color / Graphic ---

    fn red_expr() -> ExprColor {
        Expr::Node(NodeColor::Literal(typed::Color::Rgba {
            r: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
            g: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            b: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            a: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
        }))
    }

    fn blue_expr() -> ExprColor {
        Expr::Node(NodeColor::Literal(typed::Color::Rgba {
            r: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            g: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            b: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
            a: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
        }))
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
        let expr = Expr::Node(NodeColor::If(If {
            condition: Box::new(Expr::Node(NodeBool::Literal(true))),
            then_branch: Box::new(red_expr()),
            else_branch: Box::new(blue_expr()),
        }));
        assert_eq!(eval_color(&context, expr).unwrap(), red_scene());

        // if false { red } else { blue } → blue
        let expr = Expr::Node(NodeColor::If(If {
            condition: Box::new(Expr::Node(NodeBool::Literal(false))),
            then_branch: Box::new(red_expr()),
            else_branch: Box::new(blue_expr()),
        }));
        assert_eq!(eval_color(&context, expr).unwrap(), blue_scene());
    }

    #[test]
    fn test_eval_match_color_literal() {
        let context = Context::new();
        // match red { red => blue, x => x } → blue
        let expr = Expr::Node(NodeColor::Match {
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
        });
        assert_eq!(eval_color(&context, expr).unwrap(), blue_scene());
    }

    #[test]
    fn test_eval_match_color_binding() {
        let context = Context::new();
        // match red { blue => red, x => x } → red (binding arm wins)
        let expr = Expr::Node(NodeColor::Match {
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
        });
        assert_eq!(eval_color(&context, expr).unwrap(), red_scene());
    }

    #[test]
    fn test_eval_match_color_no_match() {
        let context = Context::new();
        // match red { blue => red } → err
        let expr = Expr::Node(NodeColor::Match {
            scrutinee: Box::new(red_expr()),
            arms: vec![MatchArm {
                pattern: Pattern::Literal(blue_expr()),
                guard: None,
                body: red_expr(),
            }],
        });
        assert!(eval_color(&context, expr).is_err());
    }

    #[test]
    fn test_eval_if_graphic() {
        let context = Context::new();
        let expr = Expr::Node(NodeGraphic::If(If {
            condition: Box::new(Expr::Node(NodeBool::Literal(false))),
            then_branch: Box::new(circle_expr()),
            else_branch: Box::new(rect_expr()),
        }));
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
        let expr = Expr::Node(NodeGraphic::Match {
            scrutinee: Box::new(circle_expr()),
            arms: vec![MatchArm {
                pattern: Pattern::Literal(circle_expr()),
                guard: None,
                body: rect_expr(),
            }],
        });
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
}
