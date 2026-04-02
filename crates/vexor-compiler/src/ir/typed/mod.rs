//! Typed IR nodes

use crate::ir::typed::expr::{ExprGeneric, ExprGraphic, ExprNumber, ExprString};
use std::collections::HashMap;

pub mod expr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Number,
    String,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        identifier: String,
        value: ExprGeneric,
    },
    Export {
        graphic: ExprGraphic,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub varTypes: HashMap<String, Type>,
    pub statements: Vec<Statement>,
}
