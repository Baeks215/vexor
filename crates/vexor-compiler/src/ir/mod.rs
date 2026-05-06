pub mod ast;
pub mod scene;

/// User Number type in the compiler: always a 64-bit floating point number
pub type Number = f64;

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
