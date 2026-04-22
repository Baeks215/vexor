//! Evaluator for expressions

use crate::evaluator::program::eval_assignment;
use crate::evaluator::{Context, EResult, Function, Value};
use crate::ir::Number;
use crate::ir::scene;
use crate::ir::typed;
use crate::ir::typed::expr::{
    Expr, ExprColor, ExprGeneric, ExprGraphic, ExprNumber, ExprString, NodeNumber, OpBinNumber,
};

pub fn eval_generic(context: &Context, expr: ExprGeneric) -> EResult<Value> {
    Ok(match expr {
        ExprGeneric::Number(expr) => Value::Number(eval_number(context, expr)?),
        ExprGeneric::String(expr) => Value::String(eval_string(context, expr)?),
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
        Expr::Node(literal) => Ok(literal),
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
        Expr::Node(typed::Color::Rgba { r, g, b, a }) => Ok(scene::Color::Rgba {
            r: eval_number(context, *r)?,
            g: eval_number(context, *g)?,
            b: eval_number(context, *b)?,
            a: eval_number(context, *a)?,
        }),
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
        Expr::Node(typed::Graphic::Circle { radius }) => Ok(scene::Graphic::Circle {
            radius: eval_number(context, *radius)?,
        }),
        Expr::Node(typed::Graphic::Rect { width, height }) => Ok(scene::Graphic::Rect {
            width: eval_number(context, *width)?,
            height: eval_number(context, *height)?,
        }),
        Expr::Node(typed::Graphic::Text(text)) => {
            Ok(scene::Graphic::Text(eval_string(context, *text)?))
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

        let expr = Expr::Node("hello".to_string());
        assert_eq!(eval_string(&context, expr).unwrap(), "hello");

        let expr = Expr::Variable("name".to_string());
        assert_eq!(eval_string(&context, expr).unwrap(), "vexor");
    }

    #[test]
    fn test_eval_color() {
        let context = Context::new();
        let expr = Expr::Node(typed::Color::Rgba {
            r: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
            g: Box::new(Expr::Node(NodeNumber::Literal(0.5))),
            b: Box::new(Expr::Node(NodeNumber::Literal(0.0))),
            a: Box::new(Expr::Node(NodeNumber::Literal(1.0))),
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

    #[test]
    fn test_eval_graphic() {
        let context = Context::new();

        // Circle
        let expr = Expr::Node(typed::Graphic::Circle {
            radius: Box::new(Expr::Node(NodeNumber::Literal(15.0))),
        });
        let res = eval_graphic(&context, expr).unwrap();
        assert_eq!(res, scene::Graphic::Circle { radius: 15.0 });

        // Text
        let expr = Expr::Node(typed::Graphic::Text(Box::new(Expr::Node("hi".to_string()))));
        let res = eval_graphic(&context, expr).unwrap();
        assert_eq!(res, scene::Graphic::Text("hi".to_string()));
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
    }
}
