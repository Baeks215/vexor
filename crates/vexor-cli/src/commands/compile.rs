use clap::Args;
use std::path::PathBuf;

#[derive(Args, Clone)]
pub struct CompileArgs {
    /// Path to the .vx source file
    pub input: PathBuf,
    /// Output path (file for single export, directory for multiple)
    pub output: PathBuf,
}

pub fn run(args: CompileArgs) {
    if let Err(e) = compile_to_svg(args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

pub fn compile_to_svg(args: CompileArgs) -> Result<(), String> {
    let source = std::fs::read_to_string(&args.input)
        .map_err(|e| format!("could not read '{}': {e}", args.input.display()))?;

    let exports = vexor_compiler::compile_to_svg(&source).map_err(|e| {
        format!(
            "compilation failed for '{}':\n\n{}",
            args.input.display(),
            e.format_colored()
        )
    })?;

    if exports.len() == 1 {
        // Write single export to target path
        std::fs::write(&args.output, &exports[0].data)
            .map_err(|e| format!("could not write '{}': {e}", args.output.display()))?;
    } else {
        // Write multiple exports to target directory
        std::fs::create_dir_all(&args.output).map_err(|e| {
            format!(
                "could not create directory '{}': {e}",
                args.output.display()
            )
        })?;
        for export in &exports {
            let path = args.output.join(format!("{}.svg", export.name));
            std::fs::write(&path, &export.data)
                .map_err(|e| format!("could not write '{}': {e}", path.display()))?;
        }
    }
    println!("--- Compiled successfully ---");
    Ok(())
}
