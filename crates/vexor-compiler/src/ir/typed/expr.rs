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
    Color(ExprColor),
    Graphic(ExprGraphic),
}

pub type ExprNumber = Expr<NodeNumber>;
pub type ExprString = Expr<String>;
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

// Other types are only literals for now
