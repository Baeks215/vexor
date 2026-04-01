//! Typed IR

pub mod ast;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Number,
    String,
    Color,
    Graphic,
}
