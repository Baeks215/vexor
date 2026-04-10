use crate::ir::scene::{Graphic, Scene};

mod evaluator;
mod exporter;
mod ir;
mod parser;
mod type_checker;

pub use exporter::*;

/// Compiles the given input string into [`Scene`] IR.
pub fn compile(input: &str) -> Option<Scene> {
    let parsed = parser::parse_program(input).ok()?;
    let typed = type_checker::check_program(parsed).ok()?;
    let scene = evaluator::eval_program(typed).ok()?;
    Some(scene)
}

/// Compiles input with given exporter
fn compile_to<T>(input: &str, exporter: fn(Scene) -> Vec<Export<T>>) -> Option<Vec<Export<T>>> {
    let scene = compile(input)?;
    Some(exporter(scene))
}

/// Compiles input to SVG exports
pub fn compile_to_svg(input: &str) -> Option<Vec<Export<String>>> {
    compile_to(input, export_scene_svg)
}
