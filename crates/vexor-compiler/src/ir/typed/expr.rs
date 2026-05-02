//! Typed IR nodes for expressions

use std::fmt::Debug;

use crate::ir::Number;
use crate::ir::typed::{BoolT, Color, ColorT, Graphic, GraphicT, NumberT, StringT};

pub trait SemanticType: Debug + Clone {
    /// Type used in Rust compiler
    type NativeType: Debug + Clone;
    /// Defines available operators
    type OperatorNode: Debug + Clone;
}

impl SemanticType for NumberT {
    type NativeType = Number;
    type OperatorNode = NumberOps;
}

impl SemanticType for StringT {
    type NativeType = String;
    type OperatorNode = ();
}

impl SemanticType for BoolT {
    type NativeType = bool;
    type OperatorNode = BoolOps;
}

impl SemanticType for ColorT {
    type NativeType = Color;
    type OperatorNode = ();
}

impl SemanticType for GraphicT {
    type NativeType = Graphic;
    type OperatorNode = ();
}

// Operators

#[derive(Debug, Clone)]
pub enum NumberOps {
    Arithmetic {
        op: ArithmeticOp,
        left: Box<Expr<NumberT>>,
        right: Box<Expr<NumberT>>,
    },
}

#[derive(Debug, Clone)]
pub enum BoolOps {
    Compare {
        op: CompareOp,
        left: Box<Expr<NumberT>>,
        right: Box<Expr<NumberT>>,
    },
    Logic {
        op: LogicOp,
        left: Box<Expr<BoolT>>,
        right: Box<Expr<BoolT>>,
    },
    Not(Box<Expr<BoolT>>),
}

#[derive(Debug, Clone)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum LogicOp {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum CompareOp {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Neq,
}

/// Common Expression node.
#[derive(Debug, Clone)]
pub enum Expr<T: SemanticType> {
    Literal(T::NativeType),
    Variable(String),
    Operator(T::OperatorNode),
    Field {
        object: String,
        field: String,
    },
    Call {
        function: String,
        arguments: Vec<ExprGeneric>,
    },
    If {
        condition: Box<Expr<BoolT>>,
        then_branch: Box<Expr<T>>,
        else_branch: Box<Expr<T>>,
    },
    Match {
        scrutinee: Box<Expr<T>>,
        arms: Vec<MatchArm<Expr<T>>>,
    },
}

#[derive(Debug, Clone)]
pub enum ExprGeneric {
    Number(Expr<NumberT>),
    String(Expr<StringT>),
    Bool(Expr<BoolT>),
    Color(Expr<ColorT>),
    Graphic(Expr<GraphicT>),
}

// --- Match ---

#[derive(Debug, Clone)]
pub enum Pattern<E> {
    Binding(String),
    Literal(E),
}

#[derive(Debug, Clone)]
pub struct MatchArm<E> {
    pub pattern: Pattern<E>,
    pub guard: Option<Expr<BoolT>>,
    pub body: E,
}
