//! Type resolver for expressions

use crate::ir::ast::{self, OpBin, OpUn};
use crate::ir::typed::expr::{
    ArithmeticOp, BoolOps, CompareOp, Expr, ExprGeneric, LogicOp, MatchArm, NumberOps, Pattern,
    SemanticType,
};
use crate::ir::typed::{self, BoolT, ColorT, GraphicT, NumberT, StringT, Type};
use crate::type_checker::{Constraint, Context, TResult};
use Constraint::*;

/// Checkable expressions
pub trait Checkable: SemanticType {
    /// Type check a literal node
    fn check_literal(context: &Context, literal: ast::Literal) -> TResult<Self::NativeType>;
    /// Build a binary operator node
    fn check_op_bin(
        context: &Context,
        operator: OpBin,
        left: ast::Expr,
        right: ast::Expr,
    ) -> TResult<Self::OperatorNode>;
    /// Build a unary operator node
    fn check_op_un(
        context: &Context,
        operator: OpUn,
        operand: ast::Expr,
    ) -> TResult<Self::OperatorNode>;
}

pub fn check_generic(context: &Context, ty: Type, expr: ast::Expr) -> TResult<ExprGeneric> {
    match ty {
        Type::Number => Ok(ExprGeneric::Number(check(context, expr)?)),
        Type::String => Ok(ExprGeneric::String(check(context, expr)?)),
        Type::Bool => Ok(ExprGeneric::Bool(check(context, expr)?)),
        Type::Color => Ok(ExprGeneric::Color(check(context, expr)?)),
        Type::Graphic => Ok(ExprGeneric::Graphic(check(context, expr)?)),
        Type::GType(_) => Ok(ExprGeneric::Graphic(check(context, expr)?)),
    }
}

fn check_func_args(
    context: &Context,
    function: &str,
    args: Vec<ast::Expr>,
    return_constraint: Constraint,
) -> TResult<Vec<ExprGeneric>> {
    let arg_types = context.check_function(&function, return_constraint)?;
    if arg_types.len() != args.len() {
        return Err("Invalid number of arguments".to_string());
    }
    args.into_iter()
        .zip(arg_types)
        .map(|(arg, ty)| check_generic(context, ty, arg))
        .collect::<Result<Vec<_>, _>>()
}

/// Type-checks the condition (bool) and both branches (of type E) of an if expression.
fn check_if<T: Checkable>(
    context: &Context,
    condition: ast::Expr,
    then_branch: ast::Expr,
    else_branch: ast::Expr,
) -> TResult<Expr<T>> {
    let condition = Box::new(check::<BoolT>(context, condition)?);
    let then_branch = Box::new(check(context, then_branch)?);
    let else_branch = Box::new(check(context, else_branch)?);
    Ok(Expr::If {
        condition,
        then_branch,
        else_branch,
    })
}

/// Type-checks match arms for a match whose scrutinee and body are of type E.
fn check_match_arms<T: Checkable>(
    context: &Context,
    arms: Vec<ast::MatchArm>,
) -> TResult<Vec<MatchArm<Expr<T>>>> {
    arms.into_iter()
        .map(|arm| {
            let ast::MatchArm {
                pattern,
                guard,
                body,
            } = arm;
            let (pattern, scope) = match pattern {
                ast::Pattern::Binding(name) => {
                    let scope = context.with_var(name.clone(), T::TYPE_ENUM);
                    (Pattern::Binding(name), Some(scope))
                }
                ast::Pattern::Literal(e) => (
                    Pattern::Literal(check::<T>(context, ast::Expr::Literal(e))?),
                    None,
                ),
            };
            let arm_ctx = scope.as_ref().unwrap_or(context);
            let guard = guard.map(|g| check::<BoolT>(arm_ctx, g)).transpose()?;
            let body = check::<T>(arm_ctx, body)?;
            Ok(MatchArm {
                pattern,
                guard,
                body,
            })
        })
        .collect()
}

/// Checks a match expression.
fn check_match<T: Checkable>(
    context: &Context,
    scrutinee: ast::Expr,
    arms: Vec<ast::MatchArm>,
) -> TResult<Expr<T>> {
    let scrutinee = Box::new(check::<T>(context, scrutinee)?);
    let arms = check_match_arms::<T>(context, arms)?;
    Ok(Expr::Match { scrutinee, arms })
}

/// Checks an expression using generics.
pub fn check<T: Checkable>(context: &Context, expr: ast::Expr) -> TResult<Expr<T>> {
    match expr {
        ast::Expr::Literal(lit) => T::check_literal(context, lit).map(|val| Expr::Literal(val)),
        ast::Expr::Variable(name) => {
            context.check_var(&name, Is(T::TYPE_ENUM))?;
            Ok(Expr::Variable(name))
        }
        ast::Expr::Call { function, args } => {
            let typed_args = check_func_args(context, &function, args, Is(T::TYPE_ENUM))?;
            Ok(Expr::Call {
                function,
                arguments: typed_args,
            })
        }
        ast::Expr::Binary {
            operator,
            left,
            right,
        } => {
            let op_node = T::check_op_bin(context, operator, *left, *right)?;
            Ok(Expr::Operator(op_node))
        }
        ast::Expr::Unary { operator, operand } => {
            let op_node = T::check_op_un(context, operator, *operand)?;
            Ok(Expr::Operator(op_node))
        }
        ast::Expr::Match { scrutinee, arms } => check_match(context, *scrutinee, arms),
        ast::Expr::If {
            condition,
            then_branch,
            else_branch,
        } => check_if(context, *condition, *then_branch, *else_branch),
        ast::Expr::Field { object, field } => check_field_access(context, object, field),
    }
}

macro_rules! extract_fields {
    ($fields:expr, [$($name:ident),+]) => {{
        let fields: Vec<(String, _)> = $fields;

        // Check for unexpected
        let expected = [$(stringify!($name)),+];
        for (key, _) in fields.iter() {
            if !expected.contains(&key.as_str()) {
                // TODO: Return multiple errors for each unexpected field
                return Err(format!("unexpected field: {key}"));
            }
        }

        // Extract and move fields to tupl
        let mut map: std::collections::HashMap<String, _> = fields.into_iter().collect();

        $(
            let $name = map.remove(stringify!($name))
                .ok_or_else(|| format!("missing field: {}", stringify!($name)))?;
        )+

        ($($name),+)
    }};
}

/// Checks a field access type
fn check_field_access<T: Checkable>(
    context: &Context,
    object: String,
    field: String,
) -> TResult<Expr<T>> {
    // Only Graphic types can have fields, for now
    let obj_ty = context.check_var(&object, Is(Type::Graphic))?;
    let Type::GType(obj_ty) = obj_ty else {
        return Err("Expected Graphic type".to_string());
    };
    let field_ty = match obj_ty {
        typed::GraphicType::Circle => match field.as_str() {
            "x" | "y" | "radius" => Type::Number,
            "color" => Type::Color,
            _ => return Err(format!("Unknown field {field} for Circle type")),
        },
        typed::GraphicType::Rect => match field.as_str() {
            "x" | "y" | "width" | "height" => Type::Number,
            "color" => Type::Color,
            _ => return Err(format!("Unknown field {field} for Rect type")),
        },
        typed::GraphicType::Text => match field.as_str() {
            "x" | "y" => Type::Number,
            "content" => Type::String,
            "color" => Type::Color,
            _ => return Err(format!("Unknown field {field} for Text type")),
        },
    };
    field_ty
        .satisfies(Is(T::TYPE_ENUM))
        .map(|_| Expr::Field { object, field })
}

impl Checkable for NumberT {
    fn check_literal(_: &Context, literal: ast::Literal) -> TResult<Self::NativeType> {
        match literal {
            ast::Literal::Number(n) => Ok(n),
            _ => Err("expected number literal".to_string()),
        }
    }

    fn check_op_bin(
        context: &Context,
        operator: OpBin,
        left: ast::Expr,
        right: ast::Expr,
    ) -> TResult<Self::OperatorNode> {
        let op = match operator {
            ast::OpBin::Add => ArithmeticOp::Add,
            ast::OpBin::Sub => ArithmeticOp::Sub,
            ast::OpBin::Mul => ArithmeticOp::Mul,
            ast::OpBin::Div => ArithmeticOp::Div,
            _ => return Err("Invalid operator for number".to_string()),
        };

        let left = check::<NumberT>(context, left)?;
        let right = check::<NumberT>(context, right)?;
        Ok(NumberOps::Arithmetic {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn check_op_un(_: &Context, _: OpUn, _: ast::Expr) -> TResult<Self::OperatorNode> {
        Err("Invalid operator for number".to_string())
    }
}

impl Checkable for StringT {
    fn check_literal(_: &Context, literal: ast::Literal) -> TResult<Self::NativeType> {
        match literal {
            ast::Literal::String(s) => Ok(s),
            _ => Err("expected string literal".to_string()),
        }
    }

    fn check_op_bin(
        _: &Context,
        _: OpBin,
        _: ast::Expr,
        _: ast::Expr,
    ) -> TResult<Self::OperatorNode> {
        Err("Invalid operator for string".to_string())
    }

    fn check_op_un(_: &Context, _: OpUn, _: ast::Expr) -> TResult<Self::OperatorNode> {
        Err("Invalid operator for string".to_string())
    }
}

impl Checkable for BoolT {
    fn check_literal(_: &Context, literal: ast::Literal) -> TResult<Self::NativeType> {
        match literal {
            ast::Literal::Bool(b) => Ok(b),
            _ => Err("expected bool literal".to_string()),
        }
    }

    fn check_op_bin(
        context: &Context,
        operator: OpBin,
        left: ast::Expr,
        right: ast::Expr,
    ) -> TResult<Self::OperatorNode> {
        if let Some(op) = match operator {
            ast::OpBin::And => Some(LogicOp::And),
            ast::OpBin::Or => Some(LogicOp::Or),
            _ => None,
        } {
            let l_bool = check(context, left.clone())?;
            let r_bool = check(context, right.clone())?;
            return Ok(BoolOps::Logic {
                op,
                left: Box::new(l_bool),
                right: Box::new(r_bool),
            });
        }
        let op = match operator {
            ast::OpBin::Gt => CompareOp::Gt,
            ast::OpBin::Gte => CompareOp::Gte,
            ast::OpBin::Lt => CompareOp::Lt,
            ast::OpBin::Lte => CompareOp::Lte,
            ast::OpBin::Eq => CompareOp::Eq,
            ast::OpBin::Neq => CompareOp::Neq,
            _ => return Err("Invalid operator for bool".to_string()),
        };

        let l_number = check::<NumberT>(context, left)?;
        let r_number = check::<NumberT>(context, right)?;
        Ok(BoolOps::Compare {
            op,
            left: Box::new(l_number),
            right: Box::new(r_number),
        })
    }

    fn check_op_un(
        context: &Context,
        operator: OpUn,
        operand: ast::Expr,
    ) -> TResult<Self::OperatorNode> {
        let op = match operator {
            ast::OpUn::Not => BoolOps::Not,
        };
        let operand = check(context, operand)?;
        Ok(op(Box::new(operand)))
    }
}

impl Checkable for ColorT {
    fn check_literal(context: &Context, literal: ast::Literal) -> TResult<Self::NativeType> {
        match literal {
            ast::Literal::Color(ast::Color::Rgba { r, g, b, a }) => {
                let r = Box::new(check::<NumberT>(context, *r)?);
                let g = Box::new(check::<NumberT>(context, *g)?);
                let b = Box::new(check::<NumberT>(context, *b)?);
                let a = Box::new(check::<NumberT>(context, *a)?);
                Ok(typed::Color::Rgba { r, g, b, a })
            }
            _ => Err("Expected color literal".to_string()),
        }
    }

    fn check_op_bin(
        _: &Context,
        _: OpBin,
        _: ast::Expr,
        _: ast::Expr,
    ) -> TResult<Self::OperatorNode> {
        Err("Invalid operator for color".to_string())
    }

    fn check_op_un(_: &Context, _: OpUn, _: ast::Expr) -> TResult<Self::OperatorNode> {
        Err("Invalid operator for string".to_string())
    }
}

impl Checkable for GraphicT {
    fn check_literal(context: &Context, literal: ast::Literal) -> TResult<Self::NativeType> {
        match literal {
            ast::Literal::Object(ast::Object { name, fields }) => match name.as_str() {
                "Circle" => {
                    let (x, y, radius, color) = extract_fields!(fields, [x, y, radius, color]);
                    Ok(typed::Graphic::Circle {
                        x: Box::new(check(context, x)?),
                        y: Box::new(check(context, y)?),
                        radius: Box::new(check(context, radius)?),
                        color: Box::new(check(context, color)?),
                    })
                }
                "Rect" => {
                    let (x, y, width, height, color) =
                        extract_fields!(fields, [x, y, width, height, color]);
                    Ok(typed::Graphic::Rect {
                        x: Box::new(check(context, x)?),
                        y: Box::new(check(context, y)?),
                        width: Box::new(check(context, width)?),
                        height: Box::new(check(context, height)?),
                        color: Box::new(check(context, color)?),
                    })
                }
                "Text" => {
                    let (x, y, content, color) = extract_fields!(fields, [x, y, content, color]);
                    Ok(typed::Graphic::Text {
                        x: Box::new(check(context, x)?),
                        y: Box::new(check(context, y)?),
                        content: Box::new(check(context, content)?),
                        color: Box::new(check(context, color)?),
                    })
                }
                _ => Err("Unexpected object name".to_string()),
            },
            _ => Err("Unexpected expression, expected a graphic".to_string()),
        }
    }

    fn check_op_bin(
        _: &Context,
        _: OpBin,
        _: ast::Expr,
        _: ast::Expr,
    ) -> TResult<Self::OperatorNode> {
        Err("Invalid operator for graphic".to_string())
    }

    fn check_op_un(_: &Context, _: OpUn, _: ast::Expr) -> TResult<Self::OperatorNode> {
        Err("Invalid operator for graphic".to_string())
    }
}
