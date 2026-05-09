//! Abstract Syntax Tree nodes

use crate::ir::Number;

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

/// Graphic Object
#[derive(Debug, Clone)]
pub enum Graphic {
    Circle {
        x: Box<Expr>,
        y: Box<Expr>,
        radius: Box<Expr>,
        color: Box<Expr>,
    },
    Rect {
        x: Box<Expr>,
        y: Box<Expr>,
        width: Box<Expr>,
        height: Box<Expr>,
        color: Box<Expr>,
    },
    Text {
        x: Box<Expr>,
        y: Box<Expr>,
        content: Box<Expr>,
        color: Box<Expr>,
    },
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
    Cons,
}

#[derive(Debug, Clone, Copy)]
pub enum OpUn {
    Not,
}

#[derive(Debug, Clone)]
pub enum ListLiteral {
    List(Vec<Expr>),
    Range {
        start: Box<Expr>,
        second: Option<Box<Expr>>,
        end: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(Number),
    String(String),
    Bool(bool),
    Color(Color),
    Graphic(Graphic),
    List(ListLiteral),
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
    /// Standard Function call
    Std(Std),
    /// Constant
    Const(Const),
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

// --- Standard Functions ---

#[derive(Debug, Clone)]
pub enum Std {
    // Trigonometric functions
    Rad(Box<Expr>),
    Sin(Box<Expr>),
    Cos(Box<Expr>),
    Tan(Box<Expr>),
    // List utilities
    Map {
        function: Box<Expr>,
        list: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Const {
    Pi,
}

// --- Match ---

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Expr,
    pub guard: Option<Expr>,
    pub body: Expr,
}

// --- Program ---

#[derive(Debug, Clone)]
pub struct Assignment {
    pub identifier: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub scope: Vec<Assignment>,
    pub return_expr: Expr,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub scope: Vec<Assignment>,
    pub exports: Vec<Expr>,
}
