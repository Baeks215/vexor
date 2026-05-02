//! Typed IR nodes

use crate::ir::typed::expr::{Expr, ExprGeneric};

pub mod expr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Number,
    String,
    Bool,
    Color,
    Graphic,
    GType(GraphicType),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphicType {
    Circle,
    Rect,
    Text,
}

// Marker Types
#[derive(Debug, Clone, Copy)]
pub struct NumberT;
#[derive(Debug, Clone, Copy)]
pub struct StringT;
#[derive(Debug, Clone, Copy)]
pub struct BoolT;
#[derive(Debug, Clone, Copy)]
pub struct ColorT;
#[derive(Debug, Clone, Copy)]
pub struct GraphicT;

// --- Primitives ---

/// Color Literal, typed
#[derive(Debug, Clone)]
pub enum Color {
    Rgba {
        r: Box<Expr<NumberT>>,
        g: Box<Expr<NumberT>>,
        b: Box<Expr<NumberT>>,
        a: Box<Expr<NumberT>>,
    },
}

/// Renderable graphic component, typed
#[derive(Debug, Clone)]
pub enum Graphic {
    Circle {
        x: Box<Expr<NumberT>>,
        y: Box<Expr<NumberT>>,
        radius: Box<Expr<NumberT>>,
        color: Box<Expr<ColorT>>,
    },
    Rect {
        x: Box<Expr<NumberT>>,
        y: Box<Expr<NumberT>>,
        width: Box<Expr<NumberT>>,
        height: Box<Expr<NumberT>>,
        color: Box<Expr<ColorT>>,
    },
    Text {
        x: Box<Expr<NumberT>>,
        y: Box<Expr<NumberT>>,
        content: Box<Expr<StringT>>,
        color: Box<Expr<ColorT>>,
    },
}

// --- Program ---

/// Statements, either of a function body or top-level
#[derive(Debug, Clone)]
pub struct Assignment {
    pub identifier: String,
    pub value: ExprGeneric,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub scope: Vec<Assignment>,
    pub return_expr: ExprGeneric,
}

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    pub scope: Vec<Assignment>,
    pub exports: Vec<Expr<GraphicT>>,
}
