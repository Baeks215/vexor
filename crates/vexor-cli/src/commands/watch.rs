use crate::commands::{compile, file_watch};

pub fn run(args: compile::CompileArgs) {
    let input = args.input.clone();
    let result = file_watch::watch_file(&input, |compiled| match compiled {
        Ok(exports) => match compile::write_exports(&args.output, &exports) {
            Ok(()) => println!("--- Compiled successfully ---"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Err(e) => eprintln!("Error: {e}"),
    });
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
