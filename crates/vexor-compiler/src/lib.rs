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
    use crate::ir::scene::{Color, Graphic};

    fn red() -> Color {
        Color::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    fn circle(x: f64, y: f64, radius: f64) -> Graphic {
        Graphic::Circle {
            x,
            y,
            radius,
            color: red(),
        }
    }

    fn rect(x: f64, y: f64, width: f64, height: f64) -> Graphic {
        Graphic::Rect {
            x,
            y,
            width,
            height,
            color: red(),
        }
    }

    fn text(x: f64, y: f64, content: &str) -> Graphic {
        Graphic::Text {
            x,
            y,
            content: content.to_string(),
            color: red(),
        }
    }

    const RED: &str = "color.rgb(1, 0, 0, 1)";

    #[test]
    fn test_compile_single_export() {
        let input = format!("export Circle {{ x: 0, y: 0, radius: 10, color: {RED} }}");
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], circle(0.0, 0.0, 10.0));
    }

    #[test]
    fn test_compile_with_assignment() {
        let input =
            format!("let r: number = 5\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}");
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], circle(0.0, 0.0, 5.0));
    }

    #[test]
    fn test_compile_multiple_exports() {
        let input = format!(
            "export Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}\nexport Rect {{ x: 0, y: 0, width: 2, height: 3, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 2);
        assert_eq!(scene.exports[0], circle(0.0, 0.0, 1.0));
        assert_eq!(scene.exports[1], rect(0.0, 0.0, 2.0, 3.0));
    }

    #[test]
    fn test_compile_invalid_input() {
        assert!(compile("not valid vexor code !!!").is_none());
    }

    #[test]
    fn test_compile_to_svg_single() {
        let input = format!("export Circle {{ x: 0, y: 0, radius: 10, color: {RED} }}");
        let exports = compile_to_svg(&input).expect("compile_to_svg should succeed");
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "export_0");
        assert!(exports[0].data.contains("<circle"));
    }

    #[test]
    fn test_compile_to_svg_multiple() {
        let input = format!(
            "export Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}\nexport Rect {{ x: 0, y: 0, width: 2, height: 3, color: {RED} }}"
        );
        let exports = compile_to_svg(&input).expect("compile_to_svg should succeed");
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
        let input = format!(
            "let flag: bool = true\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], circle(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_compile_with_compare() {
        let input =
            format!("let b: bool = 3 > 2\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}");
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_with_bool_function() {
        let input = format!(
            "fn cmp(a: number, b: number): bool = a > b\nlet flag: bool = cmp(5, 3)\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_rejects_compare_as_number() {
        let input = format!(
            "let x: number = 1 > 2\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        );
        assert!(compile(&input).is_none());
    }

    #[test]
    fn test_compile_with_logical_ops() {
        let input = format!(
            "let x: bool = true && !false || 1 == 1\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_rejects_logical_on_numbers() {
        let input = format!(
            "let x: bool = 1 && 2\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        );
        assert!(compile(&input).is_none());
    }

    #[test]
    fn test_compile_with_match() {
        let input = format!(
            "let x: number = 5\nlet r: number = match x {{ x if x > 10 => 100, 2 => 99, y => y + 1 }}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], circle(0.0, 0.0, 6.0));
    }

    #[test]
    fn test_compile_with_string_match() {
        let input = format!(
            "let s: string = match \"hi\" {{ \"hi\" => \"hello\", x => x }}\nexport Text {{ x: 0, y: 0, content: s, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], text(0.0, 0.0, "hello"));
    }

    #[test]
    fn test_compile_with_bool_match() {
        let input = format!(
            "let flag: bool = match true {{ true => false, x => x }}\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_with_if() {
        let input = format!(
            "let x: number = 5\nlet r: number = if x > 10 {{ 100 }} else {{ x + 1 }}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], circle(0.0, 0.0, 6.0));
    }

    #[test]
    fn test_compile_with_if_string() {
        let input = format!(
            "let s: string = if true {{ \"yes\" }} else {{ \"no\" }}\nexport Text {{ x: 0, y: 0, content: s, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], text(0.0, 0.0, "yes"));
    }

    #[test]
    fn test_compile_with_if_bool() {
        let input = format!(
            "let b: bool = if false {{ true }} else {{ false }}\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_rejects_if_non_bool() {
        let input = format!(
            "let x: number = if 1 {{ 1 }} else {{ 2 }}\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        );
        assert!(compile(&input).is_none());
    }

    #[test]
    fn test_compile_if_else_if_nesting() {
        let input = format!(
            "let x: number = 5\nlet r: number = if x > 10 {{ 100 }} else {{ if x > 3 {{ 50 }} else {{ 0 }} }}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports[0], circle(0.0, 0.0, 50.0));
    }

    #[test]
    fn test_compile_with_if_graphic() {
        let input = format!(
            "export if true {{ Circle {{ x: 0, y: 0, radius: 10, color: {RED} }} }} else {{ Rect {{ x: 0, y: 0, width: 5, height: 5, color: {RED} }} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports[0], circle(0.0, 0.0, 10.0));

        let input = format!(
            "export if false {{ Circle {{ x: 0, y: 0, radius: 10, color: {RED} }} }} else {{ Rect {{ x: 0, y: 0, width: 5, height: 5, color: {RED} }} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports[0], rect(0.0, 0.0, 5.0, 5.0));
    }

    #[test]
    fn test_compile_with_match_graphic() {
        let input = format!(
            "let g: graphic = Circle {{ x: 0, y: 0, radius: 10, color: {RED} }}\nexport match g {{ Circle {{ x: 0, y: 0, radius: 10, color: {RED} }} => Rect {{ x: 0, y: 0, width: 1, height: 2, color: {RED} }}, x => x }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports[0], rect(0.0, 0.0, 1.0, 2.0));
    }

    #[test]
    fn test_compile_with_if_color() {
        let input = format!(
            "let c: color = if true {{ color.rgb(1, 0, 0, 1) }} else {{ color.rgb(0, 0, 1, 1) }}\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
    }

    #[test]
    fn test_compile_with_function() {
        let input = format!(
            "fn double(x: number): number = x + x\nexport Circle {{ x: 0, y: 0, radius: double(5), color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], circle(0.0, 0.0, 10.0));

        let input = format!(
            "fn area(w: number, h: number): number = w * h\nexport Rect {{ x: 0, y: 0, width: area(2, 3), height: 4, color: {RED} }}"
        );
        let scene = compile(&input).expect("compile should succeed");
        assert_eq!(scene.exports.len(), 1);
        assert_eq!(scene.exports[0], rect(0.0, 0.0, 6.0, 4.0));
    }
}
