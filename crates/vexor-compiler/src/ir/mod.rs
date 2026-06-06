use std::rc::Rc;

pub mod ast;
pub mod path;
pub mod scene;

/// User Number type in the compiler: always a 64-bit floating point number
pub type Number = f64;

/// A user identifier (variable / parameter / binding name), interned as `Rc<str>`
///   to prevent duplicate allocations
pub type Ident = Rc<str>;
