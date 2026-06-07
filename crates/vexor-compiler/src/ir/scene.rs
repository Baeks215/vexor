//! High-level IR representing a scene. Common IR before render and file output.

use kurbo::Affine;
use std::rc::Rc;

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
    pub attrs: Vec<Attr>,
    pub transform: Affine,
}
impl Graphic {
    /// Creates a new graphic component with no attributes.
    pub fn new(ty: GraphicType) -> Self {
        Self {
            ty,
            attrs: vec![],
            transform: Affine::default(),
        }
    }

    /// Applies a geometric transformation to the graphic component.
    pub fn transform(self: Rc<Self>, transform: Affine) -> Rc<Self> {
        let mut g = self;
        let m = Rc::make_mut(&mut g);
        m.transform = transform * m.transform;
        g
    }

    /// Applies a transformation in the graphic's local space.
    pub fn transform_local(self: Rc<Self>, transform: Affine) -> Rc<Self> {
        let mut g = self;
        let m = Rc::make_mut(&mut g);
        m.transform = m.transform * transform;
        g
    }

    /// Appends an attribute. Later attributes of the same kind win at render time.
    pub fn with_attr(self: Rc<Self>, attr: Attr) -> Rc<Self> {
        let mut g = self;
        Rc::make_mut(&mut g).attrs.push(attr);
        g
    }
}

#[derive(Debug, Clone)]
pub enum GraphicType {
    Circle {
        radius: Number,
    },
    Ellipse {
        rx: Number,
        ry: Number,
    },
    Rect {
        width: Number,
        height: Number,
    },
    Text {
        content: String,
    },
    Path {
        path: Path,
    },
    Group {
        // Outer `Rc` shares the group; inner `Rc<Graphic>` shares each child graphic
        children: Rc<[Rc<Graphic>]>,
    },
}

/// A single attribute of a graphic component (style or identity).
#[derive(Debug, Clone)]
pub enum Attr {
    Fill(Color),
    StrokeColor(Color),
    StrokeWidth(Number),
    StrokeJoin(StrokeJoin),
    StrokeCap(StrokeCap),
    Opacity(Number),
    /// SVG `id` attribute.
    Id(String),
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
