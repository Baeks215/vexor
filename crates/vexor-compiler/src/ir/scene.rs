//! High-level IR representing a scene. Common IR before render and file output.

use crate::ir::Number;

// --- Primitives ---

/// Color Literal, typed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Rgba {
        r: Number,
        g: Number,
        b: Number,
        a: Number,
    },
}

/// Renderable graphic component, typed
#[derive(Debug, Clone, PartialEq)]
pub enum Graphic {
    Circle { radius: Number },
    Rect { width: Number, height: Number },
    Text(String),
}

// --- Scene ---

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub exports: Vec<Graphic>,
}
