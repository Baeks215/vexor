//! High-level IR representing a scene. Common IR before render and file output.

use kurbo::{Affine, BezPath};

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
    Hsla {
        h: Number,
        s: Number,
        l: Number,
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
    /// Creates a new graphic component with default attributes.
    pub fn new(ty: GraphicType) -> Self {
        Self {
            ty,
            style: Style::default(),
            transform: Affine::default(),
        }
    }

    /// Applies a geometric transformation to the graphic component.
    pub fn transform(self, transform: Affine) -> Self {
        Self {
            transform: transform * self.transform,
            ..self
        }
    }

    /// Applies a transformation in the graphic's local space.
    pub fn transform_local(self, transform: Affine) -> Self {
        Self {
            transform: self.transform * transform,
            ..self
        }
    }

    /// Applies a transformation to the style of the graphic component.
    pub fn transform_style(self, f: impl FnOnce(Style) -> Style) -> Self {
        Self {
            style: f(self.style),
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphicType {
    Circle { radius: Number },
    Rect { width: Number, height: Number },
    Text { content: String },
    Path { path: BezPath },
    Group { children: Vec<Graphic> },
}

/// Style of a graphic component
#[derive(Debug, Clone)]
pub struct Style {
    pub fill: Color,
    pub stroke: Option<Stroke>,
}
impl Style {
    pub fn with_fill(self, fill: Color) -> Self {
        Self { fill, ..self }
    }
    pub fn with_stroke(self, stroke: Stroke) -> Self {
        Self {
            stroke: Some(stroke),
            ..self
        }
    }
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

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub canvas: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub exports: Vec<Graphic>,
    pub settings: Settings,
}
