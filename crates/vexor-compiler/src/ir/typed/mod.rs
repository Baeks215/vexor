//! Typed IR nodes

use crate::ir::typed::expr::{ExprGeneric, ExprGraphic, ExprNumber, ExprString};

pub mod expr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Number,
    String,
    Bool,
    Color,
    Graphic,
}

// --- Primitives ---

/// Color Literal, typed
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Rgba {
        r: Box<ExprNumber>,
        g: Box<ExprNumber>,
        b: Box<ExprNumber>,
        a: Box<ExprNumber>,
    },
}

/// Renderable graphic component, typed
#[derive(Debug, Clone, PartialEq)]
pub enum Graphic {
    Circle {
        radius: Box<ExprNumber>,
    },
    Rect {
        width: Box<ExprNumber>,
        height: Box<ExprNumber>,
    },
    Text(Box<ExprString>),
}

// --- Program ---

/// Statements, either of a function body or top-level
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub identifier: String,
    pub value: ExprGeneric,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub scope: Vec<Assignment>,
    pub return_expr: ExprGeneric,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub scope: Vec<Assignment>,
    pub exports: Vec<ExprGraphic>,
}
