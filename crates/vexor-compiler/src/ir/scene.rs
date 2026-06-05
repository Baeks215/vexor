//! High-level IR representing a scene. Common IR before render and file output.

use kurbo::Affine;

use crate::ir::Number;
use crate::ir::path::Path;

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
    pub attr: Attr,
    pub transform: Affine,
}
impl Graphic {
    /// Creates a new graphic component with default attributes.
    pub fn new(ty: GraphicType) -> Self {
        Self {
            ty,
            attr: Attr::default(),
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

    /// Applies a transformation to the attributes of the graphic component.
    pub fn transform_attr(self, f: impl FnOnce(Attr) -> Attr) -> Self {
        Self {
            attr: f(self.attr),
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphicType {
    Circle { radius: Number },
    Rect { width: Number, height: Number },
    Text { content: String },
    Path { path: Path },
    Group { children: Vec<Graphic> },
}

/// Presentation attributes of a graphic component (style + identity).
#[derive(Debug, Clone)]
pub struct Attr {
    pub fill: Option<Color>,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<Number>,
    pub stroke_join: Option<StrokeJoin>,
    pub stroke_cap: Option<StrokeCap>,
    pub opacity: Option<Number>,
    /// Optional SVG `id` attribute.
    pub id: Option<String>,
}
impl Attr {
    pub fn with_fill(self, fill: Color) -> Self {
        Self {
            fill: Some(fill),
            ..self
        }
    }
    pub fn with_stroke_color(self, color: Color) -> Self {
        Self {
            stroke_color: Some(color),
            ..self
        }
    }
    pub fn with_stroke_width(self, width: Number) -> Self {
        Self {
            stroke_width: Some(width),
            ..self
        }
    }
    pub fn with_stroke_join(self, join: StrokeJoin) -> Self {
        Self {
            stroke_join: Some(join),
            ..self
        }
    }
    pub fn with_stroke_cap(self, cap: StrokeCap) -> Self {
        Self {
            stroke_cap: Some(cap),
            ..self
        }
    }
    pub fn with_opacity(self, opacity: Number) -> Self {
        Self {
            opacity: Some(opacity),
            ..self
        }
    }
    pub fn with_id(self, id: String) -> Self {
        Self {
            id: Some(id),
            ..self
        }
    }
}
impl Default for Attr {
    fn default() -> Self {
        Self {
            // No explicit fill; SVG defaults to black.
            fill: None,
            // No stroke
            stroke_color: None,
            stroke_width: None,
            stroke_join: None,
            stroke_cap: None,
            // Fully opaque
            opacity: None,
            // No id
            id: None,
        }
    }
}

/// Stroke line join style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

/// Stroke line cap style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

// --- Scene ---

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub canvas: (usize, usize),
    /// Number of decimal places for floats in the generated SVG.
    pub precision: usize,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            canvas: (1000, 1000), // Default canvas size
            precision: 3,         // Default decimal places for SVG output
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub exports: Vec<Graphic>,
    pub settings: Settings,
}
