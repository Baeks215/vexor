//! High-level IR representing a scene. Common IR before render and file output.

use crate::ir::Number;

// --- Primitives ---

/// Color Literal, typed
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Rgba {
        r: Number,
        g: Number,
        b: Number,
        a: Number,
    },
}

/// Renderable graphic component, typed
#[derive(Debug, Clone)]
pub struct Graphic {
    pub ty: GraphicType,
    pub style: Style,
}

#[derive(Debug, Clone)]
pub enum GraphicType {
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

/// Style of a graphic component
#[derive(Debug, Clone)]
pub struct Style {
    pub fill: Color,
    pub stroke: Option<Stroke>,
}

/// Stroke of a graphic component
#[derive(Debug, Clone)]
pub struct Stroke {
    pub color: Color,
    pub width: Number,
}

// --- Scene ---

#[derive(Debug, Clone)]
pub struct Scene {
    pub exports: Vec<Graphic>,
}
