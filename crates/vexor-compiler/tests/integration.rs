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
    compile(input).unwrap_or_else(|e| panic!("{}", e))
}

fn assert_number(expr: &str, expected: f64) {
    let input = format!("let r = {expr}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}");
    assert_eq!(ok(&input).exports[0], circle(0.0, 0.0, expected));
}

fn assert_body_number(body: &str, expr: &str, expected: f64) {
    let input =
        format!("{body}\nlet r = {expr}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}");
    assert_eq!(ok(&input).exports[0], circle(0.0, 0.0, expected));
}

fn assert_string(expr: &str, expected: &str) {
    let input = format!("let s = {expr}\nexport Text {{ x: 0, y: 0, content: s, color: {RED} }}");
    assert_eq!(ok(&input).exports[0], text(0.0, 0.0, expected));
}

fn assert_bool_compiles(expr: &str) {
    let input = format!("let b = {expr}\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}");
    ok(&input);
}

fn assert_rejects(input: &str) {
    assert!(compile(input).is_err());
}

#[test]
fn test_compile_basics() {
    let single = format!("export Circle {{ x: 0, y: 0, radius: 10, color: {RED} }}");
    assert_eq!(ok(&single).exports, vec![circle(0.0, 0.0, 10.0)]);

    let with_let = format!("let r = 5\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}");
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
    assert!(compile_to_svg("garbage @@@").is_err());
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
        "fn cmp(a, b) = a > b\nlet flag = cmp(5, 3)\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
    );
    ok(&with_fn);
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
        "let g = match 1 {{ 1 => Circle {{ x: 0, y: 0, radius: 7, color: {RED} }}, x => Rect {{ x: 0, y: 0, width: 1, height: 1, color: {RED} }} }}\nexport g"
    );
    assert_eq!(ok(&num_to_graphic).exports[0], circle(0.0, 0.0, 7.0));

    // Destructure Circle fields, use captured radius in body (7 * 2 = 14)
    let prog = format!(
        "let g = Circle {{ x: 0, y: 0, radius: 7, color: {RED} }}\nlet r = match g {{ Circle {{ x: cx, y: cy, radius: radius, color: c }} => radius * 2, y => 0 }}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}"
    );
    assert_eq!(ok(&prog).exports[0], circle(0.0, 0.0, 14.0));

    // Destructure Rect fields, use width * height (4 * 5 = 20)
    let prog = format!(
        "let b = Rect {{ x: 0, y: 0, width: 4, height: 5, color: {RED} }}\nlet r = match b {{ Rect {{ x: rx, y: ry, width: w, height: h, color: c }} => w * h, y => 0 }}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}"
    );
    assert_eq!(ok(&prog).exports[0], circle(0.0, 0.0, 20.0));

    // Guard on destructured field: radius 15 > 10 => "big"
    let prog = format!(
        "let g = Circle {{ x: 0, y: 0, radius: 15, color: {RED} }}\nlet s = match g {{ Circle {{ x: cx, y: cy, radius: r, color: c }} if r > 10 => \"big\", x => \"small\" }}\nexport Text {{ x: 0, y: 0, content: s, color: {RED} }}"
    );
    assert_eq!(ok(&prog).exports[0], text(0.0, 0.0, "big"));

    // Literal field in graphic pattern: radius must equal 5 exactly
    let prog = format!(
        "let g = Circle {{ x: 0, y: 0, radius: 5, color: {RED} }}\nlet s = match g {{ Circle {{ x: cx, y: cy, radius: 5, color: c }} => \"exact\", x => \"other\" }}\nexport Text {{ x: 0, y: 0, content: s, color: {RED} }}"
    );
    assert_eq!(ok(&prog).exports[0], text(0.0, 0.0, "exact"));

    // Multi-variant: extract x coord from whichever graphic variant matches
    let prog = format!(
        "let g = Rect {{ x: 3, y: 0, width: 1, height: 1, color: {RED} }}\nlet r = match g {{ Circle {{ x: gx, y: gy, radius: radius, color: c }} => gx, Rect {{ x: gx, y: gy, width: w, height: h, color: c }} => gx, y => 0 }}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}"
    );
    assert_eq!(ok(&prog).exports[0], circle(0.0, 0.0, 3.0));

    // Variant mismatch: Rect does not match Circle pattern, falls to catch-all
    let prog = format!(
        "let g = Rect {{ x: 0, y: 0, width: 2, height: 2, color: {RED} }}\nlet r = match g {{ Circle {{ x: cx, y: cy, radius: radius, color: c }} => radius, y => 99 }}\nexport Circle {{ x: 0, y: 0, radius: r, color: {RED} }}"
    );
    assert_eq!(ok(&prog).exports[0], circle(0.0, 0.0, 99.0));
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
        "let c = if true {{ rgb(1, 0, 0, 1) }} else {{ rgb(0, 0, 1, 1) }}\nexport Circle {{ x: 0, y: 0, radius: 1, color: {RED} }}"
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
        "let g = Circle {{ x: 0, y: 0, radius: 10, color: {RED} }}\nexport match g {{ x if true => Rect {{ x: 0, y: 0, width: 1, height: 2, color: {RED} }}, y => y }}"
    );
    assert_eq!(ok(&match_g).exports[0], rect(0.0, 0.0, 1.0, 2.0));
}

#[test]
fn test_compile_field_access() {
    let input = format!(
        "let box = Rect {{ x: 2, y: 8, width: 10, height: 5, color: {RED} }}\nlet c = Circle {{ x: box.x, y: box.y, radius: 5, color: {RED} }}\nexport c"
    );
    assert_eq!(ok(&input).exports[0], circle(2.0, 8.0, 5.0));
}

#[test]
fn test_compile_function() {
    let double = format!(
        "fn double(x) = x + x\nexport Circle {{ x: 0, y: 0, radius: double(5), color: {RED} }}"
    );
    assert_eq!(ok(&double).exports[0], circle(0.0, 0.0, 10.0));

    let area = format!(
        "fn area(w, h) = w * h\nexport Rect {{ x: 0, y: 0, width: area(2, 3), height: 4, color: {RED} }}"
    );
    assert_eq!(ok(&area).exports[0], rect(0.0, 0.0, 6.0, 4.0));
}

#[test]
fn test_compile_list() {
    // single-element extraction
    assert_number("match [42] { [x] => x, y => 0 }", 42.0);

    // multi-element, capture second
    assert_number("match [10, 20] { [a, b] => b, y => 0 }", 20.0);

    // literal prefix + capture
    assert_number("match [1, 2, 3] { [1, 2, x] => x, y => 0 }", 3.0);

    // string list with literal prefix
    assert_string(
        "match [\"hello\", \"world\"] { [\"hello\", x] => x, y => \"no\" }",
        "world",
    );

    // bool list compiles
    assert_bool_compiles("match [true, false] { [a, b] => a, y => false }");

    // cons operator builds list
    assert_number("match 1 : 2 : Nil { [a, b] => a, y => 0 }", 1.0);

    // match with cons deconstructing
    assert_number(
        "match [1, 2] { a : as => a + match as { 2 : Nil => 10, y => 0 }, y => 0 }",
        11.0,
    );

    // length mismatch falls through to catch-all
    assert_number("match [1, 2] { [a, b, c] => 99, y => 0 }", 0.0);
}

#[test]
fn test_compile_list_range() {
    // simple inclusive range [1..3] = [1, 2, 3]
    assert_number("match [1..3] { [a, b, c] => c, y => 0 }", 3.0);

    // reverse
    assert_number("match [3 .. 1] { [a, b, c] => c, y => 0 }", 1.0);

    // stepped range [1,3..10] = [1, 3, 5, 7, 9]
    assert_number("match [1, 3..10] { [a, b, c, d, e] => e, y => 0 }", 9.0);

    // stepped range in reverse
    assert_number("match [10, 8..2] { [a, b, c, d, e] => d, y => 0 }", 4.0);

    // range with variables: let x = 1, let y = 4 => [1..4] = [1, 2, 3, 4]
    assert_body_number(
        "let x = 1\nlet y = 4",
        "match [x .. y] { [a, b, c, d] => d, z => 0 }",
        4.0,
    );

    // range with expressions: [1 .. y - 1] where y = 4 => [1, 2, 3]
    assert_body_number(
        "let y = 4",
        "match [1 .. y - 1] { [a, b, c] => c, z => 0 }",
        3.0,
    );

    // stepped range with expression bound: [0, 2 .. x * 3] where x = 2 => [0, 2, 4, 6]
    assert_body_number(
        "let x = 2",
        "match [0, 2 .. x * 3] { [a, b, c, d] => d, z => 0 }",
        6.0,
    );
}

#[test]
fn test_compile_brackets() {
    // basic grouping
    assert_number("(1 + 2) * 3", 9.0);
    assert_number("1 + (2 * 3)", 7.0);

    // without brackets, mul binds tighter — same result as natural precedence
    assert_number("(2 * 3)", 6.0);

    // brackets override add-before-mul order
    assert_number("(1 + 2) * (3 + 4)", 21.0);
    assert_number("2 * (3 + 4) - 1", 13.0);

    // nested brackets
    assert_number("((2 + 3))", 5.0);
    assert_number("((1 + 2) * (2 + 1))", 9.0);

    // unary minus inside brackets
    assert_number("(-1 + 3) * 2", 4.0);
    assert_number("10 / (2 + 3)", 2.0);

    // brackets in function args
    let with_fn = format!(
        "fn add(a, b) = a + b\nexport Circle {{ x: 0, y: 0, radius: add((1 + 2), (3 + 4)), color: {RED} }}"
    );
    assert_eq!(ok(&with_fn).exports[0], circle(0.0, 0.0, 10.0));

    // comparison with brackets changing meaning
    assert_bool_compiles("(1 + 1) == 2");
    assert_bool_compiles("(3 * 3) > (2 + 2)");
}

#[test]
fn test_compile_std() {
    use std::f64::consts::PI;

    // pi constant
    assert_number("pi", PI);

    // exact-result trig at 0
    assert_number("rad(0)", 0.0);
    assert_number("sin(0)", 0.0);
    assert_number("cos(0)", 1.0);
    assert_number("tan(0)", 0.0);

    // rad on full turn
    assert_number("rad(180)", 180.0_f64.to_radians());
    assert_number("rad(360)", 360.0_f64.to_radians());

    // expression args
    assert_number("sin(1 + 2 - 3)", 0.0);
    assert_number("cos(2 * 0)", 1.0);

    // composition
    assert_number("sin(rad(0))", 0.0);
    assert_number("cos(rad(0))", 1.0);

    // pi inside expression
    assert_number("pi * 0", 0.0);
    assert_number("pi - pi", 0.0);

    // std calls inside let bindings + functions
    let with_let =
        format!("let z = sin(0)\nexport Circle {{ x: 0, y: 0, radius: z + 5, color: {RED} }}");
    assert_eq!(ok(&with_let).exports[0], circle(0.0, 0.0, 5.0));

    let with_fn = format!(
        "fn deg_sin(d) = sin(rad(d))\nexport Circle {{ x: 0, y: 0, radius: deg_sin(0) + 3, color: {RED} }}"
    );
    assert_eq!(ok(&with_fn).exports[0], circle(0.0, 0.0, 3.0));

    // pi as function arg
    let pi_arg = format!(
        "fn id(x) = x\nexport Circle {{ x: 0, y: 0, radius: id(pi) - id(pi), color: {RED} }}"
    );
    assert_eq!(ok(&pi_arg).exports[0], circle(0.0, 0.0, 0.0));
}
