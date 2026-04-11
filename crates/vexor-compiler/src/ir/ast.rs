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
}

// --- Program ---

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        ty: Type,
        identifier: String,
        value: Expr,
    },
    Export {
        graphic: Expr,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}
