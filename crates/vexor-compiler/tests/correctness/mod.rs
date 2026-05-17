use vexor_compiler::{Graphic, GraphicType, compile};
use winnow::Parser;
use winnow::ascii::{float, space0};
use winnow::combinator::{preceded, separated};

use std::fs;

// Parser for extracting expected outputs from source code comments
fn expected_outputs<'a>(input: &mut &'a str) -> winnow::Result<Vec<f64>> {
    preceded(
        ("-- OUTPUT:", space0),
        separated(1.., float::<_, f64, _>, (',', space0)),
    )
    .parse_next(input)
}

fn test_case(name: &str, mut source: &str) {
    let expected_outputs = expected_outputs(&mut source).unwrap_or_default();
    let graphics = match compile(&source) {
        Ok(g) => g.exports,
        Err(e) => panic!(
            "
Compilation failed: {name}
{e}"
        ),
    };
    let outputs = graphics
        .into_iter()
        .flat_map(|g| match g {
            Graphic {
                ty: GraphicType::Circle { radius },
                ..
            } => Some(radius),
            _ => None,
        })
        .collect::<Vec<_>>();

    if outputs != expected_outputs {
        panic!(
            "
Test failed: {name}
Expected {expected_outputs:?}, got {outputs:?}"
        );
    }
}

#[test]
fn test_correctness() {
    for entry in fs::read_dir("tests/correctness/cases").unwrap() {
        let path = entry.unwrap().path();
        let name_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let source = fs::read_to_string(&path).unwrap();
        test_case(&name_stem, &source);
    }
}

#[test]
#[ignore]
fn test_debug() {
    // For debugging a single case
    let name = "conditional";

    let source = fs::read_to_string(format!("tests/correctness/cases/{name}.vx")).unwrap();
    test_case(name, &source);
}
