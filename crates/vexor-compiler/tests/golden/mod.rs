use vexor_compiler::{Export, compile_to_svg};

use std::fs;

#[test]
fn test_all_goldens() {
    for entry in fs::read_dir("tests/golden/compilation/input").unwrap() {
        let path = entry.unwrap().path();
        let name_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let source = fs::read_to_string(&path).unwrap();
        let outputs = compile_to_svg(&source).unwrap();

        match outputs.as_slice() {
            // single output
            [Export { data, .. }] => {
                check_golden(&name_stem, data);
            }
            // multiple outputs
            outputs => {
                for Export { name, data } in outputs {
                    let name = format!("{name_stem}-{name}");
                    check_golden(&name, data);
                }
            }
        }
    }
}

#[test]
#[ignore]
fn test_debug() {
    // For debugging a single case
    let name = "group";

    let source = fs::read_to_string(format!("tests/golden/compilation/input/{name}.vx")).unwrap();
    compile_to_svg(&source).unwrap();
}

fn check_golden(name: &str, output: &str) {
    let path = format!("tests/golden/compilation/output/{name}.svg");
    let path = std::path::Path::new(&path);

    if path.exists() {
        let expected = fs::read_to_string(path).unwrap();
        assert_eq!(expected, output, "golden output mismatch for {name}");
    } else {
        // first run
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, output).unwrap();
        println!("created golden output: {name}");
    }
}
