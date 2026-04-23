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
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Neq,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpUn {
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Binding(String),
    Literal(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    LNumber(Number),
    LString(String),
    LBool(bool),
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
    Unary {
        operator: OpUn,
        operand: Box<Expr>,
    },
    /// Function call
    Call {
        function: String,
        args: Vec<Expr>,
    },
    /// Match expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
}

// --- Program ---

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub ty: Type,
    pub identifier: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub scope: Vec<Assignment>,
    pub return_expr: (Expr, Type),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub scope: Vec<Assignment>,
    pub exports: Vec<Expr>,
}
