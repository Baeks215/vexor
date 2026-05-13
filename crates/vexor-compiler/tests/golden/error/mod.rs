use std::io::Write;

use goldenfile::Mint;
use vexor_compiler::compile_to_svg;

#[test]
fn test_golden_error() {
    let mut mint = Mint::new("tests/golden/error/output");

    let mut entries: Vec<_> = std::fs::read_dir("tests/golden/error/input")
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();

    for path in entries {
        let name_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let source = std::fs::read_to_string(&path).unwrap();
        let err = match compile_to_svg(&source) {
            Ok(_) => panic!("{name_stem}: expected compilation error"),
            Err(e) => e,
        };

        let mut file = mint.new_goldenfile(format!("{name_stem}.err")).unwrap();
        write!(file, "{err}").unwrap();
    }
}
