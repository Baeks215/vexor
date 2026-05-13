use std::io::Write;

use goldenfile::Mint;
use vexor_compiler::{Export, compile_to_svg};

#[test]
fn test_golden_compilation() {
    let mut mint = Mint::new("tests/golden/compilation/output");

    let mut entries: Vec<_> = std::fs::read_dir("tests/golden/compilation/input")
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();

    for path in entries {
        let name_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let source = std::fs::read_to_string(&path).unwrap();
        let outputs = match compile_to_svg(&source) {
            Ok(outputs) => outputs,
            Err(e) => {
                panic!("Expected compilation to succeed\n{e}")
            }
        };

        match outputs.as_slice() {
            [Export { data, .. }] => {
                let mut file = mint.new_goldenfile(format!("{name_stem}.svg")).unwrap();
                write!(file, "{data}").unwrap();
            }
            outputs => {
                for Export { name, data } in outputs {
                    let mut file = mint
                        .new_goldenfile(format!("{name_stem}-{name}.svg"))
                        .unwrap();
                    write!(file, "{data}").unwrap();
                }
            }
        }
    }
}

#[test]
#[ignore]
fn test_debug() {
    let name = "group";
    let source =
        std::fs::read_to_string(format!("tests/golden/compilation/input/{name}.vx")).unwrap();
    compile_to_svg(&source).unwrap();
}
