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

    #[test]
    fn test_compile_with_bool_assignment() {
        let input = "let flag: bool = true\nexport circle(1)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], Graphic::Circle { radius: 1.0 });
    }

    #[test]
    fn test_compile_with_compare() {
        let input = "let b: bool = 3 > 2\nexport circle(1)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_with_bool_function() {
        let input = "fn cmp(a: number, b: number): bool = a > b\nlet flag: bool = cmp(5, 3)\nexport circle(1)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_rejects_compare_as_number() {
        // Assigning a comparison to a number-typed var must fail type check
        assert!(compile("let x: number = 1 > 2\nexport circle(1)").is_none());
    }

    #[test]
    fn test_compile_with_logical_ops() {
        let input = "let x: bool = true && !false || 1 == 1\nexport circle(1)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_rejects_logical_on_numbers() {
        assert!(compile("let x: bool = 1 && 2\nexport circle(1)").is_none());
    }

    #[test]
    fn test_compile_with_match() {
        let input = "let x: number = 5\nlet r: number = match x { x if x > 10 => 100, 2 => 99, y => y + 1 }\nexport circle(r)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], Graphic::Circle { radius: 6.0 });
    }

    #[test]
    fn test_compile_with_string_match() {
        let input = "let s: string = match \"hi\" { \"hi\" => \"hello\", x => x }\nexport text(s)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], Graphic::Text("hello".to_string()));
    }

    #[test]
    fn test_compile_with_bool_match() {
        let input = "let flag: bool = match true { true => false, x => x }\nexport circle(1)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_with_if() {
        let input =
            "let x: number = 5\nlet r: number = if x > 10 { 100 } else { x + 1 }\nexport circle(r)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], Graphic::Circle { radius: 6.0 });
    }

    #[test]
    fn test_compile_with_if_string() {
        let input = "let s: string = if true { \"yes\" } else { \"no\" }\nexport text(s)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], Graphic::Text("yes".to_string()));
    }

    #[test]
    fn test_compile_with_if_bool() {
        let input = "let b: bool = if false { true } else { false }\nexport circle(1)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_rejects_if_non_bool() {
        assert!(compile("let x: number = if 1 { 1 } else { 2 }\nexport circle(1)").is_none());
    }

    #[test]
    fn test_compile_if_else_if_nesting() {
        let input = "let x: number = 5\nlet r: number = if x > 10 { 100 } else { if x > 3 { 50 } else { 0 } }\nexport circle(r)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports[0], Graphic::Circle { radius: 50.0 });
    }

    #[test]
    fn test_compile_with_function() {
        let input = "fn double(x: number): number = x + x\nexport circle(double(5))";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], Graphic::Circle { radius: 10.0 });

        let input = "fn area(w: number, h: number): number = w * h\nexport rect(area(2, 3), 4)";
        let scene = compile(input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(
            scene.exports[0],
            Graphic::Rect {
                width: 6.0,
                height: 4.0,
            }
        );
    }
}
