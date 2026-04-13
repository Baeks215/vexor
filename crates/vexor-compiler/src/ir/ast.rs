//! Abstract Syntax Tree nodes

use crate::ir::{Number, typed::Type};

// --- Primitives ---

/// Color symbol: in various representations
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Rgba {
        r: Box<Expr>,
        g: Box<Expr>,
        b: Box<Expr>,
        a: Box<Expr>,
    },
}

/// Renderable graphic component
#[derive(Debug, Clone, PartialEq)]
pub enum Graphic {
    Circle { radius: Box<Expr> },
    Rect { width: Box<Expr>, height: Box<Expr> },
    Text(Box<Expr>),
}

// --- Expressions ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpBin {
    Add,
    Sub,
    Mul,
    Div,
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    LNumber(Number),
    LString(String),
    LColor(Color),
    LGraphic(Graphic),
    // Variable
    Variable(String),
    // Expressions with operators
    Binary {
        operator: OpBin,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Function call
    Call {
        function: String,
        args: Vec<Expr>,
    },
}

// --- Program ---

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        ty: Type,
        identifier: String,
        value: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub body: Vec<Statement>,
    pub return_expr: (Expr, Type),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub statements: Vec<Statement>,
    pub exports: Vec<Expr>,
}
