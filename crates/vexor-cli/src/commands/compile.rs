use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct CompileArgs {
    /// Path to the .vx source file
    pub input: PathBuf,
    /// Output path (file for single export, directory for multiple)
    pub output: PathBuf,
}

pub fn run(args: CompileArgs) {
    let source = match std::fs::read_to_string(&args.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{}': {e}", args.input.display());
            std::process::exit(1);
        }
    };

    let exports = match vexor_compiler::compile_to_svg(&source) {
        Ok(exports) => exports,
        Err(e) => {
            eprintln!(
                "error: compilation failed for '{}':\n\n{}",
                args.input.display(),
                e
            );
            std::process::exit(1);
        }
    };

    if exports.len() == 1 {
        // Write single export to target path
        if let Err(e) = std::fs::write(&args.output, &exports[0].data) {
            eprintln!("error: could not write '{}': {e}", args.output.display());
            std::process::exit(1);
        }
    } else {
        // Write multiple exports to target directory
        if let Err(e) = std::fs::create_dir_all(&args.output) {
            eprintln!(
                "error: could not create directory '{}': {e}",
                args.output.display()
            );
            std::process::exit(1);
        }
        for export in &exports {
            let path = args.output.join(format!("{}.svg", export.name));
            if let Err(e) = std::fs::write(&path, &export.data) {
                eprintln!("error: could not write '{}': {e}", path.display());
                std::process::exit(1);
            }
        }
    }
}
