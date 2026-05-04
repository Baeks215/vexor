//! Abstract Syntax Tree nodes

use crate::ir::{Number, typed::Type};

// --- Primitives ---

/// Color symbol: in various representations
#[derive(Debug, Clone)]
pub enum Color {
    Rgba {
        r: Box<Expr>,
        g: Box<Expr>,
        b: Box<Expr>,
        a: Box<Expr>,
    },
}

/// Object with fields
#[derive(Debug, Clone)]
pub struct Object {
    pub name: String,
    pub fields: Vec<(String, Expr)>,
}

// --- Expressions ---

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum OpUn {
    Not,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Binding(String),
    Literal(Literal),
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(Number),
    String(String),
    Bool(bool),
    Color(Color),
    Object(Object),
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    Literal(Literal),
    // Variable
    Variable(String),
    // Field access
    Field {
        object: String,
        field: String,
    },
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
    /// If expression
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}

// --- Program ---

#[derive(Debug, Clone)]
pub struct Assignment {
    pub ty: Type,
    pub identifier: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
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
