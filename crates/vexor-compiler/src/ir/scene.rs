//! High-level IR representing a scene. Common IR before render and file output.

use crate::ir::Number;

// --- Primitives ---

/// Marker Types used to annotate generics
pub mod marker {
    #[derive(Debug, Clone, Copy)]
    pub struct Any;
    #[derive(Debug, Clone, Copy)]
    pub struct Number;
    #[derive(Debug, Clone, Copy)]
    pub struct String;
    #[derive(Debug, Clone, Copy)]
    pub struct Bool;
    #[derive(Debug, Clone, Copy)]
    pub struct Color;
    #[derive(Debug, Clone, Copy)]
    pub struct Graphic;
}

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
    Circle {
        x: Number,
        y: Number,
        radius: Number,
        color: Color,
    },
    Rect {
        x: Number,
        y: Number,
        width: Number,
        height: Number,
        color: Color,
    },
    Text {
        x: Number,
        y: Number,
        content: String,
        color: Color,
    },
}

// --- Scene ---

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub exports: Vec<Graphic>,
}
