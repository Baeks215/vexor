use crate::commands::bench;
use clap::Args;
use std::path::{Path, PathBuf};
use vexor_compiler::SvgExport;

#[derive(Args, Clone)]
pub struct CompileArgs {
    /// Path to the .vx source file
    pub input: PathBuf,
    /// Output path (file for single export, directory for multiple)
    pub output: PathBuf,
    /// Print compile time and memory usage
    #[arg(short, long)]
    pub stats: bool,
}

pub fn run(args: CompileArgs) {
    if let Err(e) = compile_to_svg(args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

pub fn compile_to_svg(args: CompileArgs) -> Result<(), String> {
    let (result, report) = compile_file(&args.input, args.stats);
    let exports = result?;
    write_exports(&args.output, &exports)?;
    println!("--- Compiled successfully ---");
    if let Some(report) = &report {
        println!("{report}");
    }
    Ok(())
}

/// Read `input` and compile it to SVG exports, formatting any error to a String.
/// When `stats` is set the compile is measured and a [`bench::BenchReport`]
/// returned; otherwise it runs directly with no timing or allocation tracking.
pub fn compile_file(
    input: &Path,
    stats: bool,
) -> (Result<SvgExport, String>, Option<bench::BenchReport>) {
    let compile = || {
        let source = std::fs::read_to_string(input)
            .map_err(|e| format!("could not read '{}': {e}", input.display()))?;

        vexor_compiler::compile_to_svg(&source).map_err(|e| {
            format!(
                "compilation failed for '{}':\n\n{}",
                input.display(),
                e.format_colored()
            )
        })
    };

    if stats {
        let (res, report) = bench::measure(compile);
        (res, Some(report))
    } else {
        (compile(), None)
    }
}

/// Write exports to `output`: a single export goes to the file path, multiple
/// exports go into `output` as a directory, one `<name>.svg` per export.
pub fn write_exports(
    output: &Path,
    exports: &[vexor_compiler::Export<String>],
) -> Result<(), String> {
    if exports.len() == 1 {
        // Write single export to target path
        std::fs::write(output, &exports[0].data)
            .map_err(|e| format!("could not write '{}': {e}", output.display()))?;
    } else {
        // Write multiple exports to target directory
        std::fs::create_dir_all(output)
            .map_err(|e| format!("could not create directory '{}': {e}", output.display()))?;
        for export in exports {
            let path = output.join(format!("{}.svg", export.name));
            std::fs::write(&path, &export.data)
                .map_err(|e| format!("could not write '{}': {e}", path.display()))?;
        }
    }
    Ok(())
}
