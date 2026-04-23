//! Typed IR nodes for expressions

use crate::ir::Number;
use crate::ir::typed::{Color, Graphic};

/// Common Expression node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<T> {
    Variable(String),
    Node(T),
    Call {
        function: String,
        arguments: Vec<ExprGeneric>,
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
pub type ExprString = Expr<String>;
pub type ExprBool = Expr<NodeBool>;
pub type ExprColor = Expr<Color>;
pub type ExprGraphic = Expr<Graphic>;

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

// Other types are only literals for now
