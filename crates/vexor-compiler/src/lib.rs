use crate::ir::scene::Scene;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::scene::Graphic;

    #[test]
    fn test_compile_single_export() {
        let input = "export circle(10)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], Graphic::Circle { radius: 10.0 });
    }

    #[test]
    fn test_compile_with_assignment() {
        let input = "let r: number = 5\nexport circle(r)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], Graphic::Circle { radius: 5.0 });
    }

    #[test]
    fn test_compile_multiple_exports() {
        let input = "export circle(1)\nexport rect(2, 3)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 2);
        assert_eq!(scene.exports[0], Graphic::Circle { radius: 1.0 });
        assert_eq!(
            scene.exports[1],
            Graphic::Rect {
                width: 2.0,
                height: 3.0,
            }
        );
    }

    #[test]
    fn test_compile_invalid_input() {
        assert!(compile("not valid vexor code !!!").is_none());
    }

    #[test]
    fn test_compile_to_svg_single() {
        let input = "export circle(10)";
        let exports = compile_to_svg(input).expect("compile_to_svg should succeed");
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "export_0");
        assert!(exports[0].data.contains("<circle"));
    }

    #[test]
    fn test_compile_to_svg_multiple() {
        let input = "export circle(1)\nexport rect(2, 3)";
        let exports = compile_to_svg(input).expect("compile_to_svg should succeed");
        assert_eq!(exports.len(), 2);
        assert_eq!(exports[0].name, "export_0");
        assert_eq!(exports[1].name, "export_1");
        assert!(exports[0].data.contains("<circle"));
        assert!(exports[1].data.contains("<rect"));
    }

    #[test]
    fn test_compile_to_svg_invalid_input() {
        assert!(compile_to_svg("garbage @@@").is_none());
    }
}
