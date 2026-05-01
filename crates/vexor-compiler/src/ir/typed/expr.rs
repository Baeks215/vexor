//! Typed IR nodes for expressions

use crate::ir::Number;
use crate::ir::typed::{Color, Graphic};

/// Common Expression node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<T> {
    Variable(String),
    Field {
        object: String,
        field: String,
    },
    Node(T),
    Call {
        function: String,
        arguments: Vec<ExprGeneric>,
    },
    If {
        condition: Box<ExprBool>,
        then_branch: Box<Expr<T>>,
        else_branch: Box<Expr<T>>,
    },
    Match {
        scrutinee: Box<Expr<T>>,
        arms: Vec<MatchArm<Expr<T>>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprGeneric {
    Number(ExprNumber),
    String(ExprString),
    Bool(ExprBool),
    Color(ExprColor),
    Graphic(ExprGraphic),
}

pub type ExprNumber = Expr<NodeNumber>;
pub type ExprString = Expr<NodeString>;
pub type ExprBool = Expr<NodeBool>;
pub type ExprColor = Expr<NodeColor>;
pub type ExprGraphic = Expr<NodeGraphic>;

// --- Match ---

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern<E> {
    Binding(String),
    Literal(E),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm<E> {
    pub pattern: Pattern<E>,
    pub guard: Option<ExprBool>,
    pub body: E,
}

// --- Number Type ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpBinNumber {
    Add,
    Sub,
    Mul,
    Div,
}

/// Number Node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeNumber {
    Literal(Number),
    // Expressions with operators
    Binary {
        operator: OpBinNumber,
        left: Box<ExprNumber>,
        right: Box<ExprNumber>,
    },
}

// --- String Type ---

/// String Node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeString {
    Literal(String),
}

// --- Bool Type ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCompare {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Neq,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpBinBool {
    And,
    Or,
    Eq,
    Neq,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpUnBool {
    Not,
}

/// Bool Node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeBool {
    Literal(bool),
    Compare {
        operator: OpCompare,
        left: Box<ExprNumber>,
        right: Box<ExprNumber>,
    },
    Binary {
        operator: OpBinBool,
        left: Box<ExprBool>,
        right: Box<ExprBool>,
    },
    Unary {
        operator: OpUnBool,
        operand: Box<ExprBool>,
    },
}

// --- Color Type ---

/// Color Node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeColor {
    Literal(Color),
}

// --- Graphic Type ---

/// Graphic Node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeGraphic {
    Literal(Graphic),
}
