use vexor_compiler::{Color, Graphic, Scene, compile, compile_to_svg};

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

const RED: &str = "rgb(1, 0, 0, 1)";

fn ok(input: &str) -> Scene {
    compile(input).expect("compile should succeed")
}

fn assert_number(expr: &str, expected: f64) {
    let input =
        format!("let r: Number = {expr}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}");
    assert_eq!(ok(&input).exports[0], circle(0.0, 0.0, expected));
}

fn assert_string(expr: &str, expected: &str) {
    let input =
        format!("let s: String = {expr}\nexport Text {{ x: 0, y: 0, content: s, color: {RED} }}");
    assert_eq!(ok(&input).exports[0], text(0.0, 0.0, expected));
}

fn assert_bool_compiles(expr: &str) {
    let input =
        format!("let b: Bool = {expr}\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}");
    ok(&input);
}

fn assert_rejects(input: &str) {
    assert!(compile(input).is_none());
}

#[test]
fn test_compile_basics() {
    let single = format!("export Circle {{ x: 0, y: 0, radius: 10, color: {RED} }}");
    assert_eq!(ok(&single).exports, vec![circle(0.0, 0.0, 10.0)]);

    let with_let =
        format!("let r: Number = 5\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}");
    assert_eq!(ok(&with_let).exports, vec![circle(0.0, 0.0, 5.0)]);

    let multi = format!(
        "export Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}\nexport Rect {{ x: 0, y: 0, width: 2, height: 3, color: {RED} }}"
    );
    assert_eq!(
        ok(&multi).exports,
        vec![circle(0.0, 0.0, 1.0), rect(0.0, 0.0, 2.0, 3.0)],
    );
}

#[test]
fn test_compile_invalid_input() {
    assert_rejects("not valid vexor code !!!");
    assert!(compile_to_svg("garbage @@@").is_none());
}

#[test]
fn test_compile_to_svg() {
    let single = format!("export Circle {{ x: 0, y: 0, radius: 10, color: {RED} }}");
    let exports = compile_to_svg(&single).expect("compile_to_svg should succeed");
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].name, "export_0");
    assert!(exports[0].data.contains("<circle"));

    let multi = format!(
        "export Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}\nexport Rect {{ x: 0, y: 0, width: 2, height: 3, color: {RED} }}"
    );
    let exports = compile_to_svg(&multi).expect("compile_to_svg should succeed");
    assert_eq!(exports.len(), 2);
    assert_eq!(exports[0].name, "export_0");
    assert_eq!(exports[1].name, "export_1");
    assert!(exports[0].data.contains("<circle"));
    assert!(exports[1].data.contains("<rect"));
}

#[test]
fn test_compile_bool_exprs() {
    for expr in [
        "true",
        "3 > 2",
        "true && !false || 1 == 1",
        "1 == 1 && 2 != 3",
    ] {
        assert_bool_compiles(expr);
    }

    let with_fn = format!(
        "fn cmp(a: Number, b: Number): Bool = a > b\nlet flag: Bool = cmp(5, 3)\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
    );
    ok(&with_fn);
}

#[test]
fn test_compile_rejects_invalid_typing() {
    let cases = [
        format!("let x: Number = 1 > 2\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"),
        format!("let x: Bool = 1 && 2\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"),
        format!(
            "let x: Number = if 1 {{ 1 }} else {{ 2 }}\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
        ),
    ];
    for input in cases {
        assert_rejects(&input);
    }
}

#[test]
fn test_compile_match() {
    // Same-type scrutinee and body
    assert_number("match 5 { x if x > 10 => 100, 2 => 99, y => y + 1 }", 6.0);
    assert_string("match \"hi\" { \"hi\" => \"hello\", x => x }", "hello");
    assert_bool_compiles("match true { true => false, x => x }");

    // Cross-type: scrutinee type differs from body type
    assert_string(
        "match 5 { x if x > 10 => \"big\", x if x > 0 => \"small\", y => \"zero\" }",
        "small",
    );
    assert_number("match true { true => 100, x => 0 }", 100.0);
    assert_bool_compiles("match \"yes\" { \"yes\" => true, x => false }");

    let num_to_graphic = format!(
        "let g: Graphic = match 1 {{ 1 => Circle {{ x: 0, y: 0, radius: 7, color: {RED} }}, x => Rect {{ x: 0, y: 0, width: 1, height: 1, color: {RED} }} }}\nexport g"
    );
    assert_eq!(ok(&num_to_graphic).exports[0], circle(0.0, 0.0, 7.0));
}

#[test]
fn test_compile_if() {
    assert_number("if 5 > 10 { 100 } else { 5 + 1 }", 6.0);
    assert_number(
        "if 5 > 10 { 100 } else { if 5 > 3 { 50 } else { 0 } }",
        50.0,
    );
    assert_string("if true { \"yes\" } else { \"no\" }", "yes");
    assert_bool_compiles("if false { true } else { false }");

    let if_color = format!(
        "let c: Color = if true {{ rgb(1, 0, 0, 1) }} else {{ rgb(0, 0, 1, 1) }}\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
    );
    ok(&if_color);
}

#[test]
fn test_compile_export_conditional() {
    let if_true = format!(
        "export if true {{ Circle {{ x: 0, y: 0, radius: 10, color: {RED} }} }} else {{ Rect {{ x: 0, y: 0, width: 5, height: 5, color: {RED} }} }}"
    );
    assert_eq!(ok(&if_true).exports[0], circle(0.0, 0.0, 10.0));

    let if_false = format!(
        "export if false {{ Circle {{ x: 0, y: 0, radius: 10, color: {RED} }} }} else {{ Rect {{ x: 0, y: 0, width: 5, height: 5, color: {RED} }} }}"
    );
    assert_eq!(ok(&if_false).exports[0], rect(0.0, 0.0, 5.0, 5.0));

    let match_g = format!(
        "let g: Graphic = Circle {{ x: 0, y: 0, radius: 10, color: {RED} }}\nexport match g {{ x if true => Rect {{ x: 0, y: 0, width: 1, height: 2, color: {RED} }}, y => y }}"
    );
    assert_eq!(ok(&match_g).exports[0], rect(0.0, 0.0, 1.0, 2.0));
}

#[test]
fn test_compile_field_access() {
    let input = format!(
        "let box: Rect = Rect {{ x: 2, y: 8, width: 10, height: 5, color: {RED} }}\nlet c: Circle = Circle {{ x: box.x, y: box.y, radius: 5, color: {RED} }}\nexport c"
    );
    assert_eq!(ok(&input).exports[0], circle(2.0, 8.0, 5.0));
}

#[test]
fn test_compile_function() {
    let double = format!(
        "fn double(x: Number): Number = x + x\nexport Circle {{ x: 0, y: 0, radius: double(5), color: {RED} }}"
    );
    assert_eq!(ok(&double).exports[0], circle(0.0, 0.0, 10.0));

    let area = format!(
        "fn area(w: Number, h: Number): Number = w * h\nexport Rect {{ x: 0, y: 0, width: area(2, 3), height: 4, color: {RED} }}"
    );
    assert_eq!(ok(&area).exports[0], rect(0.0, 0.0, 6.0, 4.0));
}
