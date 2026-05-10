//! High-level IR representing a scene. Common IR before render and file output.

use kurbo::Affine;

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
    pub transform: Affine,
}
impl Graphic {
    pub fn new(ty: GraphicType) -> Self {
        Self {
            ty,
            style: Style::default(),
            transform: Affine::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphicType {
    Circle { radius: Number },
    Rect { width: Number, height: Number },
    Text { content: String },
}

/// Style of a graphic component
#[derive(Debug, Clone)]
pub struct Style {
    pub fill: Color,
    pub stroke: Option<Stroke>,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            // Black fill
            fill: Color::Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            // No stroke
            stroke: None,
        }
    }
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
