use vexor_compiler::{Graphic, compile, compile_to_svg};

fn compiles(input: &str) -> Vec<Graphic> {
    compile(input).unwrap_or_else(|e| panic!("{}", e)).exports
}

fn rejects(input: &str) {
    assert!(compile(input).is_err());
}

#[test]
fn test_compile() {
    let single = format!(
        "
val r = 10  // My comment, ignore this
export Circle(r)"
    );
    matches!(compiles(&single).remove(0), Graphic { .. });

    let multi = format!(
        "
    export Circle(1)
    export Rect(2, 3)"
    );
    matches!(
        compiles(&multi).as_slice(),
        [Graphic { .. }, Graphic { .. }]
    );
}

#[test]
fn test_compile_invalid_input() {
    rejects("not valid vexor code !!!");
    rejects("let a = 2"); // No exports
}

#[test]
fn test_compile_to_svg() {
    let single = format!("export Circle(10)");
    let exports = compile_to_svg(&single).expect("compile_to_svg should succeed");
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].name, "export_0");
    assert!(exports[0].data.contains("<circle"));

    let multi = format!("export Circle(1)\nexport Rect(2, 3)");
    let exports = compile_to_svg(&multi).expect("compile_to_svg should succeed");
    assert_eq!(exports.len(), 2);
    assert_eq!(exports[0].name, "export_0");
    assert_eq!(exports[1].name, "export_1");
    assert!(exports[0].data.contains("<circle"));
    assert!(exports[1].data.contains("<rect"));
}
