//! Typed Abstract Syntax Tree nodes

use super::Type;
use crate::ir::ast;
use std::collections::HashMap;

// --- Expressions ---

/// Typed Expression
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub expr: ast::Expr,
    pub ty: Type,
}

// --- Primitives ---

/// Renderable graphic component
#[derive(Debug, Clone, PartialEq)]
pub enum Graphic {
    Circle { radius: Box<Expr> },
    Rect { width: Box<Expr>, height: Box<Expr> },
    Text(Box<Expr>),
}

// --- Program ---

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment { identifier: String, value: Expr },
    Export { graphic: Expr },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub varTypes: HashMap<String, Type>,
    pub statements: Vec<Statement>,
}
