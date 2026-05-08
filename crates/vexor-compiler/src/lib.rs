mod evaluator;
mod exporter;
mod ir;
mod parser;

pub use exporter::*;
pub use ir::scene::{Color, Graphic, Scene};

/// Compiler error
type CError = String;

/// Result type for evaluation
type CResult<O> = Result<O, CError>;

/// Compiles the given input string into [`Scene`] IR.
pub fn compile(input: &str) -> CResult<Scene> {
    let ast = parser::parse_program(input).map_err(|e| e.to_string())?;
    let scene = evaluator::eval_program(ast).map_err(|e| format!("Evaluation error: {}", e))?;
    Ok(scene)
}

/// Compiles input with given exporter
fn compile_to<T>(input: &str, exporter: fn(Scene) -> Vec<Export<T>>) -> CResult<Vec<Export<T>>> {
    let scene = compile(input)?;
    Ok(exporter(scene))
}

/// Compiles input to SVG exports
pub fn compile_to_svg(input: &str) -> CResult<Vec<Export<String>>> {
    compile_to(input, export_scene_svg)
}
